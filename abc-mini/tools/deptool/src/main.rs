use anyhow::{anyhow, bail, Context, Error, Result};
use cl::EntityKind;
use clang as cl;
use clang::sonar::{find_functions, Declaration};
use clang::source::SourceRange;
use clang::token::{Token, TokenKind};
use clang::{Clang, Entity};
use clap::{Args, Parser, Subcommand};
use fnv::FnvHashMap;
use serde::Serialize;

use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

struct DirectiveTokenPrinter<'a, 'tu>(&'a Token<'tu>);

impl fmt::Debug for DirectiveTokenPrinter<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let location = self.0.get_location().get_file_location();
        let file = location
            .file
            .map(|file| file.get_path())
            .unwrap_or(PathBuf::from("<unknown>".to_string()));
        f.debug_struct("Directive")
            .field("contents", &self.0.get_spelling())
            .field("file", &file)
            .field("line", &location.line)
            .finish()
    }
}

macro_rules! parse_ctx {
    ($token:expr, $trivia:expr) => {
        || {
            format!(
                "Failed to parse {:#?}\nwith trivia {:#?}",
                $crate::DirectiveTokenPrinter($token),
                $trivia
            )
        }
    };
    ($token:expr) => {
        || {
            format!(
                "Failed to parse {:#?}",
                $crate::DirectiveTokenPrinter($token)
            )
        }
    };
}

const DIRECTIVE_MAGIC: &str = "// ABC_MINI";

fn iter_raw_directives(range: SourceRange<'_>) -> impl Iterator<Item = Token<'_>> {
    range.tokenize().into_iter().filter(|token| {
        token.get_kind() == TokenKind::Comment && token.get_spelling().starts_with(DIRECTIVE_MAGIC)
    })
}

#[derive(Debug, Serialize)]
struct FieldDecl {
    name: String,
    ty: String,
}

#[derive(Debug, Serialize)]
struct StructDecl {
    name: String,
    fields: Vec<FieldDecl>,
}

fn parse_struct_decl(node: &Entity<'_>) -> StructDecl {
    assert_eq!(node.get_kind(), EntityKind::StructDecl);
    let name = node
        .get_name()
        .expect("struct declaration should have a name");
    let fields = node
        .get_children()
        .into_iter()
        .filter_map(|node| {
            if node.get_kind() == EntityKind::FieldDecl {
                let name = node
                    .get_name()
                    .expect("struct field declaration should have a name");
                let ty = node
                    .get_type()
                    .expect("struct field declaration should have a type")
                    .get_display_name();
                Some(FieldDecl { name, ty })
            } else {
                None
            }
        })
        .collect();
    StructDecl { name, fields }
}

#[derive(Clone, Debug, Serialize)]
struct ParamDecl {
    name: String,
    ty: String,
}

#[derive(Clone, Debug, Serialize)]
struct FuncDecl {
    name: String,
    return_ty: String,
    params: Vec<ParamDecl>,
}

fn parse_fn_param(node: Entity<'_>) -> ParamDecl {
    assert_eq!(node.get_kind(), EntityKind::ParmDecl);
    let name = node
        .get_name()
        .expect("function parameter should have a name");
    let ty = node
        .get_type()
        .expect("function parameter should have a type")
        .get_display_name();
    ParamDecl { name, ty }
}

fn parse_fn_signature(decl: Declaration<'_>) -> FuncDecl {
    let return_ty = decl
        .entity
        .get_result_type()
        .expect("function should have a return type")
        .get_display_name();
    let params = decl
        .entity
        .get_arguments()
        .expect("function should have arguments")
        .into_iter()
        .map(parse_fn_param)
        .collect();
    FuncDecl {
        name: decl.name,
        return_ty,
        params,
    }
}

#[derive(Debug, Serialize)]
enum Directive {
    AliasOf {
        typename: String,
        alias: StructDecl,
    },
    BasedOn {
        filename: String,
        commit_sha: String,
    },
    DefinedIn {
        filename: String,
        signature: FuncDecl,
    },
    DefinedInList {
        filename: String,
        signatures: Vec<FuncDecl>,
    },
    DefinedInEnd,
}

fn get_nextline_tokens<'tu>(root: &Entity<'tu>, start: &Token<'tu>) -> Vec<Token<'tu>> {
    let sourcefile = root
        .get_display_name()
        .expect("root entity should have a display name");
    let next_line = start.get_location().get_file_location().line + 1;
    root.get_range()
        .expect("root entity should have a range")
        .tokenize()
        .into_iter()
        .filter_map(|token| {
            let loc = token.get_location().get_file_location();
            match (loc.file, loc.line) {
                (Some(file), line) if file.get_path().display().to_string() == sourcefile => {
                    Some((line, token))
                }
                _ => None,
            }
        })
        .skip_while(|(line, _)| line < &next_line)
        .take_while(|(line, _)| line == &next_line)
        .map(|(_, token)| token)
        .collect()
}

fn parse_alias_of(root: &Entity<'_>, start: &Token<'_>, typename: String) -> Result<Directive> {
    let ctx = parse_ctx!(start);
    let tokens = get_nextline_tokens(root, start);
    let struct_decl = root
        .get_translation_unit()
        .annotate(&tokens)
        .into_iter()
        .find_map(|node| match node {
            Some(node) if node.get_kind() == EntityKind::StructDecl => Some(node),
            _ => None,
        })
        .ok_or(anyhow!("expected struct declaration"))
        .with_context(ctx)?;
    Ok(Directive::AliasOf {
        typename,
        alias: parse_struct_decl(&struct_decl),
    })
}

fn parse_defined_in(root: &Entity<'_>, start: &Token<'_>, filename: String) -> Result<Directive> {
    let ctx = parse_ctx!(start);
    let tokens = get_nextline_tokens(root, start);
    let nodes = root
        .get_translation_unit()
        .annotate(&tokens)
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let decl = find_functions(nodes)
        .next()
        .ok_or(anyhow!("expected function declaration"))
        .with_context(ctx)?;
    Ok(Directive::DefinedIn {
        filename,
        signature: parse_fn_signature(decl),
    })
}

#[derive(Debug)]
enum DirectiveKind {
    AliasOf,
    BasedOn,
    DefinedIn,
    DefinedInStart,
    DefinedInEnd,
}

type Trivia = Vec<String>;

fn parse_directive_kind(token: &Token<'_>) -> Result<(DirectiveKind, Trivia)> {
    let ctx = parse_ctx!(token);
    assert_eq!(token.get_kind(), TokenKind::Comment);
    let spelling = token.get_spelling();
    let mut parts = spelling.split(':').map(|part| part.trim());
    let magic = parts
        .next()
        .ok_or(anyhow!("missing magic"))
        .with_context(ctx)?;
    assert_eq!(magic, DIRECTIVE_MAGIC);
    let directive = parts
        .next()
        .ok_or(anyhow!("missing directive"))
        .with_context(ctx)?;
    let trivia: Trivia = parts.map(String::from).collect();
    match directive {
        "Alias-of" => Ok((DirectiveKind::AliasOf, trivia)),
        "Based-on" => Ok((DirectiveKind::BasedOn, trivia)),
        "Defined-in" => Ok((DirectiveKind::DefinedIn, trivia)),
        "Defined-in-start" => Ok((DirectiveKind::DefinedInStart, trivia)),
        "Defined-in-end" => Ok((DirectiveKind::DefinedInEnd, trivia)),
        _ => Err(anyhow!("unknown directive \"{}\"", directive)).with_context(ctx),
    }
}

fn parse_defined_in_list(
    root: &Entity<'_>,
    start: &Token<'_>,
    filename: String,
) -> Result<Directive> {
    let ctx = parse_ctx!(start);
    let location = start.get_location();
    let search_range = SourceRange::new(
        location,
        root.get_range()
            .ok_or(anyhow!("root entity should have a range"))
            .with_context(ctx)?
            .get_end(),
    );
    let scan_end = search_range
        .tokenize()
        .into_iter()
        .filter(|token| token.get_kind() == TokenKind::Comment)
        .find_map(|token| match parse_directive_kind(&token) {
            Ok((DirectiveKind::DefinedInEnd, _)) => Some(token.get_location()),
            _ => None,
        })
        .ok_or(anyhow!("unmatched Defined-in-start"))
        .with_context(ctx)?;
    let scan_range = SourceRange::new(location, scan_end);
    let scan_space = scan_range.tokenize();
    let nodes: Vec<_> = root
        .get_translation_unit()
        .annotate(&scan_space)
        .into_iter()
        .flatten()
        .collect();
    let signatures = cl::sonar::find_functions(nodes)
        .map(parse_fn_signature)
        .collect();
    Ok(Directive::DefinedInList {
        filename,
        signatures,
    })
}

fn parse_directive(root: &Entity<'_>, token: &Token<'_>) -> Result<Directive> {
    let (kind, trivia) = parse_directive_kind(token)?;
    let ctx = parse_ctx!(token, trivia);
    match kind {
        DirectiveKind::AliasOf => {
            let typename = trivia
                .first()
                .ok_or(anyhow!("missing alias typename"))
                .with_context(ctx)?;
            parse_alias_of(root, token, typename.clone())
        }
        DirectiveKind::BasedOn => {
            let (filename, commit_sha) = trivia
                .first()
                .ok_or(anyhow!("missing based on file"))
                .with_context(ctx)?
                .split_once(',')
                .ok_or(anyhow!("expected (<filename>, <commit_sha>)"))
                .with_context(ctx)?;
            Ok(Directive::BasedOn {
                filename: String::from(filename),
                commit_sha: String::from(commit_sha),
            })
        }
        DirectiveKind::DefinedIn => {
            let filename = trivia
                .first()
                .ok_or(anyhow!("missing defined in filename"))
                .with_context(ctx)?;
            parse_defined_in(root, token, filename.clone())
        }
        DirectiveKind::DefinedInStart => {
            let filename = trivia
                .first()
                .ok_or(anyhow!("missing defined in filename"))
                .with_context(ctx)?;
            parse_defined_in_list(root, token, filename.clone())
        }
        DirectiveKind::DefinedInEnd => Ok(Directive::DefinedInEnd),
    }
}

#[derive(Debug, PartialEq)]
enum ParseState {
    Start,
    DefinedInBegin,
}

fn parse_directives<'tu, Ts>(root: &Entity<'tu>, raw_directives: Ts) -> Result<Vec<Directive>>
where
    Ts: Iterator<Item = Token<'tu>>,
{
    let mut ds = Vec::new();
    let mut state = ParseState::Start;
    for token in raw_directives {
        let ctx = parse_ctx!(&token);
        let directive = parse_directive(root, &token)?;
        match (&state, &directive) {
            (ParseState::Start, Directive::DefinedInList { .. }) => {
                state = ParseState::DefinedInBegin;
            }
            (ParseState::Start, Directive::DefinedInEnd) => {
                return Err(anyhow!("unmatched Defined-in-end")).with_context(ctx);
            }
            (ParseState::DefinedInBegin, Directive::DefinedInList { .. }) => {
                return Err(anyhow!("nesting Defined-in-start is not allowed")).with_context(ctx);
            }
            (ParseState::DefinedInBegin, Directive::DefinedInEnd) => {
                state = ParseState::Start;
                continue;
            }
            _ => {}
        }
        ds.push(directive);
    }
    assert_eq!(state, ParseState::Start);
    Ok(ds)
}

#[derive(Args)]
struct ClangArgs {
    #[arg(required = true)]
    source_files: Vec<PathBuf>,
    #[arg(short = 'I')]
    include_dirs: Vec<PathBuf>,
    #[arg(name = "CLANG_ARGS", last = true)]
    extra: Vec<String>,
}

#[derive(Subcommand)]
enum CliCmd {
    ScanDump {
        #[command(flatten)]
        clang_args: ClangArgs,
    },
    WriteHooks {
        #[command(flatten)]
        clang_args: ClangArgs,
        #[arg(long)]
        payload_srcdir: Option<PathBuf>,
    },
}

impl CliCmd {
    fn clang_args(&self) -> &ClangArgs {
        match self {
            Self::ScanDump { clang_args } => clang_args,
            Self::WriteHooks { clang_args, .. } => clang_args,
        }
    }
}

fn scan_directives(tu: &cl::TranslationUnit) -> Result<Vec<Directive>> {
    let root = tu.get_entity();
    let range = root
        .get_range()
        .ok_or(anyhow!("root entity should not be empty"))?;
    parse_directives(&root, iter_raw_directives(range))
}

struct Rewrites<'a>(FnvHashMap<&'a str, &'a str>);

impl Rewrites<'_> {
    fn rewrite(&self, ty: &str) -> String {
        let mut parts = ty.split_whitespace().map(|part| {
            if let Some(replacement) = self.0.get(&part) {
                *replacement
            } else {
                part
            }
        });
        let mut builder = parts
            .next()
            .expect("type should have at least one part")
            .to_string();
        builder.extend(parts.map(|part| format!(" {}", part)));
        builder
    }
}

impl<'a> From<&'a Vec<Directive>> for Rewrites<'a> {
    fn from(directives: &'a Vec<Directive>) -> Self {
        Self(FnvHashMap::from_iter(directives.iter().filter_map(
            |directive| match directive {
                Directive::AliasOf { typename, alias } => {
                    Some((alias.name.as_str(), typename.as_str()))
                }
                _ => None,
            },
        )))
    }
}

#[derive(Debug)]
struct FuncHooks {
    sourcefile: String,
    functions: Vec<FuncDecl>,
}

#[derive(Clone, Copy)]
enum IncludeTypes {
    Yes,
    No,
}

#[derive(Clone, Copy)]
enum PrependComma {
    Yes,
    No,
}

fn format_parameter(
    parameter: &ParamDecl,
    rewrites: &Rewrites,
    include_types: IncludeTypes,
    prepend_comma: PrependComma,
) -> String {
    let prefix = match prepend_comma {
        PrependComma::Yes => ", ",
        PrependComma::No => "",
    };
    match include_types {
        IncludeTypes::Yes => {
            let spacing = if parameter.ty.ends_with('*') { "" } else { " " };
            format!(
                "{prefix}{}{spacing}{}",
                rewrites.rewrite(&parameter.ty),
                parameter.name
            )
        }
        IncludeTypes::No => {
            format!("{prefix}{}", parameter.name)
        }
    }
}

fn render_parameter_pack<'p, Ps>(
    mut parameters: Ps,
    rewrites: &Rewrites,
    include_types: IncludeTypes,
) -> String
where
    Ps: Iterator<Item = &'p ParamDecl>,
{
    let mut builder = String::new();
    let first = parameters.next();
    if let Some(first) = first {
        builder = format_parameter(first, rewrites, include_types, PrependComma::No);
        builder.extend(parameters.map(|parameter| {
            format_parameter(parameter, rewrites, include_types, PrependComma::Yes)
        }));
    }
    builder
}

fn render_function_declaration(function: &FuncDecl, rewrites: &Rewrites) -> String {
    let return_ty = rewrites.rewrite(&function.return_ty);
    let spacing = if return_ty.ends_with('*') { "" } else { " " };
    let params = render_parameter_pack(function.params.iter(), rewrites, IncludeTypes::Yes);
    format!("{return_ty}{spacing}{}({params})", function.name)
}

fn render_hook_invocation(function: &FuncDecl, rewrites: &Rewrites) -> String {
    let prefix = match function.return_ty.as_str() {
        "void" => "",
        _ => "return ",
    };
    let params = render_parameter_pack(function.params.iter(), rewrites, IncludeTypes::No);
    let (_, name) = function
        .name
        .split_once(HOOK_MAGIC)
        .expect("function name should start with hook magic");
    format!("{prefix}{}({params})", name)
}

fn render_function_hook(function: &FuncDecl, rewrites: &Rewrites) -> Vec<String> {
    let declaration = format!("{} {{", render_function_declaration(function, rewrites));
    let invocation = format!("  {};", render_hook_invocation(function, rewrites));
    vec![declaration, invocation, "}".to_string()]
}

const HOOK_MAGIC: &str = "AbcMini__";

fn generate_hooks(directives: &Vec<Directive>) -> Vec<FuncHooks> {
    let mut hooks = FnvHashMap::<&str, Vec<FuncDecl>>::default();
    for directive in directives {
        match directive {
            Directive::DefinedIn {
                filename,
                signature,
            } => {
                if signature.name.starts_with(HOOK_MAGIC) {
                    hooks.entry(filename).or_default().push(signature.clone());
                }
            }
            Directive::DefinedInList {
                filename,
                signatures,
            } => {
                hooks
                    .entry(filename)
                    .or_default()
                    .extend(signatures.iter().filter_map(|signature| {
                        if signature.name.starts_with(HOOK_MAGIC) {
                            Some(signature.clone())
                        } else {
                            None
                        }
                    }));
            }
            _ => {}
        }
    }
    hooks
        .into_iter()
        .filter_map(|(sourcefile, signatures)| {
            if signatures.is_empty() {
                None
            } else {
                Some(FuncHooks {
                    sourcefile: sourcefile.to_string(),
                    functions: signatures,
                })
            }
        })
        .collect()
}

const HOOK_HEADER: &str = "// AUTO-GENERATED BY ABC-MINI DEPTOOL -- DO NOT MODIFY";
const HOOK_FOOTER: &str = "// END AUTO-GENERATED BY ABC-MINI DEPTOOL";

fn render_payload(hooks: &FuncHooks, rewrites: &Rewrites) -> String {
    let mut payload = format!("{HOOK_HEADER}\n");
    for line in hooks
        .functions
        .iter()
        .flat_map(|function| render_function_hook(function, rewrites))
    {
        payload.push_str(&format!("{line}\n"));
    }
    payload.push_str(&format!("{HOOK_FOOTER}\n"));
    payload
}

const HOOKDIR: &str = "hooks";

fn parse_include(line: &str) -> Result<&str> {
    let line = line.trim();
    match line.split_once("#include") {
        Some((_, file)) => file
            .trim()
            .strip_prefix('"')
            .and_then(|file| file.strip_suffix('"'))
            .ok_or(anyhow!("expected #include \"file.h\"")),
        None => {
            bail!("expected #include");
        }
    }
}

enum IncludeFound {
    Yes,
    No,
}

fn deliver_hooks(hooks: &FuncHooks, rewrites: &Rewrites, payload_dir: &Path) -> Result<()> {
    let target = payload_dir.join(hooks.sourcefile.clone());
    if !target.exists() {
        bail!("{} does not exist", target.display());
    }
    let target_name = target
        .file_name()
        .expect("target should have a valid filename");
    let payload = render_payload(hooks, rewrites);
    let hookfile = payload_dir.join(HOOKDIR).join(target_name);
    fs::create_dir_all(
        hookfile
            .parent()
            .expect("hook file should have a parent directory"),
    )?;
    fs::write(hookfile, payload)?;
    let target_content = fs::read_to_string(&target)?;
    let target_lines = target_content.split('\n');
    let mut hooks_start: Option<usize> = None;
    let mut hooks_end: Option<usize> = None;
    let mut include_found = IncludeFound::No;
    let hookfile_include = format!(
        "{}/{}",
        HOOKDIR,
        target_name
            .to_str()
            .expect("target should have a valid filename")
    );
    for (i, line) in target_lines.enumerate() {
        let ctx = || format!("Failed to parse {}:{}", target.display(), i + 1);
        match line.trim() {
            HOOK_HEADER => {
                if hooks_start.is_some() {
                    return Err(anyhow!("duplicate hooks header")).with_context(ctx);
                }
                hooks_start = Some(i);
            }
            HOOK_FOOTER => {
                if hooks_end.is_some() {
                    return Err(anyhow!("duplicate hooks footer")).with_context(ctx);
                }
                if hooks_start.is_none() {
                    return Err(anyhow!("unmatched hooks footer")).with_context(ctx);
                }
                hooks_end = Some(i);
                break;
            }
            line if hooks_start.is_some() => {
                if parse_include(line)? == hookfile_include {
                    include_found = IncludeFound::Yes;
                }
            }
            _ => {}
        }
    }
    match (hooks_start, hooks_end, include_found) {
        (Some(_), Some(_), IncludeFound::Yes) => {}
        (Some(_), Some(hooks_end), IncludeFound::No) => {
            //
            todo!()
        }
        (None, None, IncludeFound::No) => {
            let sep = if target_content.ends_with('\n') {
                ""
            } else {
                "\n"
            };
            fs::write(
                target,
                format!(
                    "{target_content}{sep}{HOOK_HEADER}\n#include \"{hookfile_include}\"\n{HOOK_FOOTER}\n",
                ),
            )?;
        }
        (Some(i), None, _) => {
            return Err(anyhow!("unmatched hooks header"))
                .with_context(|| format!("Failed to parse {}:{}", target.display(), i + 1));
        }
        (None, Some(_), _) | (None, None, IncludeFound::Yes) => {
            unreachable!();
        }
    }
    Ok(())
}

fn handle_command(cmd: CliCmd) -> Result<()> {
    let working_dir = env::current_dir()?;
    let clang_args = cmd
        .clang_args()
        .include_dirs
        .iter()
        .map(|dir| format!("-I{}", dir.display()))
        .chain(cmd.clang_args().extra.clone())
        .collect::<Vec<_>>();
    let clang = Clang::new().map_err(Error::msg)?;
    let index = cl::Index::new(&clang, false, false);
    for source in cmd.clang_args().source_files.iter() {
        if !source.exists() {
            bail!("{} does not exist", source.display());
        }
        let tu = index
            .parser(source.clone())
            .arguments(&clang_args)
            .parse()
            .with_context(|| format!("Failed to parse {}", source.display()))?;
        match cmd {
            CliCmd::ScanDump { .. } => {
                let directives = scan_directives(&tu)?;
                println!("{}: {:#?}", source.display(), directives);
            }
            CliCmd::WriteHooks {
                ref payload_srcdir, ..
            } => {
                let payload_srcdir = payload_srcdir.as_ref().unwrap_or(&working_dir);
                let directives = scan_directives(&tu)?;
                let rewrites = Rewrites::from(&directives);
                for hooks in generate_hooks(&directives) {
                    deliver_hooks(&hooks, &rewrites, payload_srcdir)?;
                }
            }
        }
    }
    Ok(())
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: CliCmd,
}

fn main() -> Result<()> {
    handle_command(Cli::parse().command)
}
