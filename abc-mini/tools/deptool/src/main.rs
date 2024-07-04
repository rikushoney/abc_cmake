use anyhow::{anyhow, bail, Context, Error, Result};
use cl::EntityKind;
use clang as cl;
use clang::sonar::{find_functions, Declaration};
use clang::source::SourceRange;
use clang::token::{Token, TokenKind};
use clang::{Clang, Entity};
use clap::Parser;
use serde::Serialize;

use std::fmt;
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
struct Field {
    name: String,
    ty: String,
}

#[derive(Debug, Serialize)]
struct StructDecl {
    name: String,
    fields: Vec<Field>,
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
                Some(Field { name, ty })
            } else {
                None
            }
        })
        .collect();
    StructDecl { name, fields }
}

#[derive(Debug, Serialize)]
struct FnParam {
    name: String,
    ty: String,
}

#[derive(Debug, Serialize)]
struct FnSignature {
    name: String,
    return_ty: String,
    params: Vec<FnParam>,
}

fn parse_fn_param(node: Entity<'_>) -> FnParam {
    assert_eq!(node.get_kind(), EntityKind::ParmDecl);
    let name = node
        .get_name()
        .expect("function parameter should have a name");
    let ty = node
        .get_type()
        .expect("function parameter should have a type")
        .get_display_name();
    FnParam { name, ty }
}

fn parse_fn_signature(decl: Declaration<'_>) -> FnSignature {
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
    FnSignature {
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
        signature: FnSignature,
    },
    DefinedInList {
        filename: String,
        signatures: Vec<FnSignature>,
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
    let ctx = || {
        format!(
            "Failed to parse {:#?} with trivia {:#?}",
            DirectiveTokenPrinter(token),
            trivia
        )
    };
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

fn parse_directives<'tu, Ts>(root: &Entity<'tu>, raw_directives: Ts) -> Result<Vec<Directive>>
where
    Ts: Iterator<Item = Token<'tu>>,
{
    let mut ds = Vec::new();
    let mut defined_in_end_required = false;
    for token in raw_directives {
        let ctx = || format!("Failed to parse {:#?}", DirectiveTokenPrinter(&token));
        let directive = parse_directive(root, &token)?;
        match directive {
            Directive::DefinedInList { .. } => {
                if defined_in_end_required {
                    return Err(anyhow!("nesting Defined-in-start is not allowed"))
                        .with_context(ctx);
                }
                defined_in_end_required = true;
            }
            Directive::DefinedInEnd => {
                if !defined_in_end_required {
                    return Err(anyhow!("unmatched Defined-in-end")).with_context(ctx);
                }
                defined_in_end_required = false;
                continue;
            }
            _ => {}
        }
        ds.push(directive);
    }
    assert!(!defined_in_end_required);
    Ok(ds)
}

#[derive(Parser)]
struct Cli {
    search_dir: PathBuf,
    #[arg(short = 'I')]
    include_dirs: Vec<PathBuf>,
    #[arg(last = true)]
    clang_args: Vec<String>,
}

const CPP_SOURCE_EXTS: &[&str] = &["cpp"];

fn is_cpp_src(file: &Path) -> bool {
    if !file.is_file() {
        return false;
    }
    let file_ext = file
        .extension()
        .map(|ext| ext.to_str().expect("should be valid unicode").to_string())
        .unwrap_or(String::new());
    CPP_SOURCE_EXTS.contains(&file_ext.as_str())
}

fn glob_cpp_sources(rootdir: &Path) -> Result<Vec<PathBuf>> {
    assert!(rootdir.is_dir());
    let mut sources = Vec::new();
    let mut dir_stack = vec![rootdir.to_path_buf()];
    while let Some(dir) = dir_stack.pop() {
        for file in dir.read_dir()? {
            let file = file?;
            let file_type = file.file_type()?;
            let file = file.path();
            if file_type.is_dir() {
                dir_stack.push(file);
            } else if is_cpp_src(&file) {
                sources.push(file)
            }
        }
    }
    Ok(sources)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if !cli.search_dir.exists() {
        bail!("{} does not exist", cli.search_dir.display());
    }
    if !cli.search_dir.is_dir() {
        bail!("{} is not a directory", cli.search_dir.display());
    }
    let cpp_sources = glob_cpp_sources(&cli.search_dir)?;
    let clang_args: Vec<String> = cli
        .include_dirs
        .iter()
        .map(|dir| format!("-I{}", dir.display()))
        .chain(cli.clang_args)
        .collect();
    let clang = Clang::new().map_err(Error::msg)?;
    let index = cl::Index::new(&clang, false, false);
    for source in cpp_sources {
        let tu = index
            .parser(source.clone())
            .arguments(&clang_args)
            .parse()?;
        let root = tu.get_entity();
        let range = root
            .get_range()
            .ok_or(anyhow!("root entity should have a range"))?;
        let directives = parse_directives(&root, iter_raw_directives(range))?;
        println!("{}: {:#?}", source.display(), directives);
    }
    Ok(())
}
