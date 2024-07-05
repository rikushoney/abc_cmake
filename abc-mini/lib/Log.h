#ifndef _ABC_MINI_LOG_H_
#define _ABC_MINI_LOG_H_

#include "DiagnosticsImpl.h"

#include <algorithm>
#include <format>
#include <source_location>
#include <string>
#include <string_view>
#include <utility>

namespace abc_mini {
namespace detail {
inline std::string quote_if_necessary(std::string_view text) {
  constexpr std::string_view whitespace = " \t\r\n";
  const auto contains_whitespace = [whitespace](char ch) {
    return std::ranges::any_of(whitespace.cbegin(), whitespace.cend(),
                               [ch](char ws) { return ch == ws; });
  };
  if (std::ranges::any_of(text.cbegin(), text.cend(), contains_whitespace)) {
    return std::format("\"{}\"", text);
  } else {
    return std::string{text};
  }
}

template <typename... TArgs>
std::string format_message(std::string_view format, TArgs... args) {
  return std::vformat(format,
                      std::make_format_args(std::forward<TArgs>(args)...));
}
} // namespace detail

template <typename... TArgs> struct emit_fault {
  emit_fault(
      std::string_view format, TArgs &&...args,
      std::source_location source_location = std::source_location::current()) {
    auto &handlers = FaultHandlerStack::handlers();
    for (auto handler : handlers) {
      auto message =
          detail::format_message(format, std::forward<TArgs>(args)...);
#ifdef ABC_MINI_EMIT_DEBUG_INFO
      const auto file = detail::quote_if_necessary(source_location.file_name());
      const auto func =
          detail::quote_if_necessary(source_location.function_name());
      const auto line = source_location.line();
      message = std::format("{}:{}:{}: {}", file, func, line, message);
#endif
      handler(message.c_str());
    }
  }
};

template <typename... TArgs>
emit_fault(std::string_view format, TArgs &&...) -> emit_fault<TArgs...>;

} // namespace abc_mini

#endif
