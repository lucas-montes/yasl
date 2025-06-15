#include "scanner.h"
#include "common.h"
#include <string.h>

void initScanner(Scanner *scanner, const char *source) {
  scanner->start = source;
  scanner->current = source;
  scanner->line = 1;
}

static bool isAtEnd(Scanner *scanner) { return *scanner->current == '\0'; }

static Token makeToken(Scanner *scanner, TokenType type) {
  Token token;
  token.type = type;
  token.start = scanner->start;
  token.lenght = (int)(scanner->current - scanner->start);
  token.line = scanner->line;
  return token;
}

static Token errorToken(Scanner *scanner, const char *message) {
  Token token;
  token.type = TOKEN_ERROR;
  token.start = message;
  token.lenght = (int)strlen(message);
  token.line = scanner->line;
  return token;
}

static char advance(Scanner *scanner) {
  scanner->current++;
  return scanner->current[-1]; // equivalent to *(ptr - 1)
}

static bool match(Scanner *scanner, char expected) {
  if (isAtEnd(scanner) || *scanner->current != expected)
    return false;
  scanner->current++;
  return true;
}

static char peek(Scanner *scanner) { return *scanner->current; }

static char peekNext(Scanner *scanner) {
  if (isAtEnd(scanner))
    return '\0';
  return scanner->current[1];
}

static void skipWhitespace(Scanner *scanner) {
  for (;;) {
    char c = *scanner->current;
    switch (c) {
    case ' ':
    case '\r':
    case '\t':
      advance(scanner);
      break;
    case '\n':
      scanner->line++;
      advance(scanner);
      break;
    case '/':
      if (peekNext(scanner) == '/') { // Single-line comment
        while (peek(scanner) != '\n' && !isAtEnd(scanner)) {
          advance(scanner);
        }
      } else if (peekNext(scanner) == '*') { // Multi-line comment
        advance(scanner);                    // Consume the '/'
        advance(scanner);                    // Consume the '*'
        while (!(peek(scanner) == '*' && peekNext(scanner) == '/') &&
               !isAtEnd(scanner)) {
          if (peek(scanner) == '\n')
            scanner->line++;
          advance(scanner);
        }
        if (!isAtEnd(scanner)) {
          advance(scanner); // Consume the '*'
          advance(scanner); // Consume the '/'
        }
      } else {
        return; // Not a comment, exit the loop
      }
      break;
    default:
      return; // Not whitespace or comment, exit the loop
    }
  }
}

static Token string(Scanner *scanner){
  while (peek(scanner) != '"' && !isAtEnd(scanner)) {
    if (peek(scanner) == '\n') scanner->line++;
    advance(scanner);
  }

  if (isAtEnd(scanner)) {
    return errorToken(scanner, "Unterminated string.");
  }

  // The closing quote.
  advance(scanner);
  return makeToken(scanner, TOKEN_STRING);
}

Token scanToken(Scanner *scanner) {
  skipWhitespace(scanner);
  scanner->start = scanner->current;

  if (isAtEnd(scanner)) {
    return makeToken(scanner, TOKEN_EOF);
  }

  char c = advance(scanner);
  switch (c) {
  case '(':
    return makeToken(scanner, TOKEN_LEFT_PAREN);
  case ')':
    return makeToken(scanner, TOKEN_RIGHT_PAREN);
  case '{':
    return makeToken(scanner, TOKEN_LEFT_BRACE);
  case '}':
    return makeToken(scanner, TOKEN_RIGHT_BRACE);
  case ',':
    return makeToken(scanner, TOKEN_COMMA);
  case '.':
    return makeToken(scanner, TOKEN_DOT);
  case '-':
    return makeToken(scanner, TOKEN_MINUS);
  case '+':
    return makeToken(scanner, TOKEN_PLUS);
  case ';':
    return makeToken(scanner, TOKEN_SEMICOLON);
  case '*':
    return makeToken(scanner, TOKEN_STAR);
  case '/':
    return makeToken(scanner, TOKEN_SLASH);
  case '!':
    return makeToken(scanner,
                     match(scanner, '=') ? TOKEN_BANG_EQUAL : TOKEN_BANG);
  case '=':
    return makeToken(scanner,
                     match(scanner, '=') ? TOKEN_EQUAL_EQUAL : TOKEN_EQUAL);
  case '>':
    return makeToken(scanner,
                     match(scanner, '=') ? TOKEN_GREATER_EQUAL : TOKEN_GREATER);
  case '<':
    return makeToken(scanner,
                     match(scanner, '=') ? TOKEN_LESS_EQUAL : TOKEN_LESS);
  case '"': return string(scanner);

    return errorToken(scanner, "Unexpected character");
  }
}
