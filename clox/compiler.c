#include "compiler.h"
#include "scanner.h"
#include <stdio.h>
#include <stdlib.h>

#ifdef DEBUG_PRINT_CODE
#include "debug.h"
#endif

typedef struct {
  Token current;
  Token previous;
  bool hadError;
  bool panicMode;
} Parser;

typedef enum {
  PREC_NONE,
  PREC_ASSIGNMENT, // =
  PREC_OR,         // or
  PREC_AND,        // and
  PREC_EQUALITY,   // == !=
  PREC_COMPARISON, // < > <= >=
  PREC_TERM,       // + -
  PREC_FACTOR,     // * /
  PREC_UNARY,      // ! -
  PREC_CALL,       // . ()
  PREC_PRIMARY
} Precedence;

static void errorAt(Parser *parser, Token *token, const char *message) {
  if (parser->panicMode)
    return;
  parser->panicMode = true;
  fprintf(stderr, "[line %d] Error\n", token->line);
  if (token->type == TOKEN_EOF) {
    fprintf(stderr, " at end");
  } else if (token->type == TOKEN_ERROR) {
    // TODO
  } else {
    fprintf(stderr, " at '%.*s'", token->length, token->start);
  }
  fprintf(stderr, ": %s\n", message);
  parser->hadError = true;
}

static void error(Parser *parser, const char *message) {
  errorAt(parser, &parser->previous, message);
}

static void errorAtCurrent(Parser *parser, const char *message) {
  errorAt(parser, &parser->current, message);
}

static void advance(Parser *parser, Scanner *scanner) {
  parser->previous = parser->current;
  for (;;) {
    parser->current = scanToken(scanner);
    if (parser->current.type != TOKEN_ERROR)
      break;

    errorAtCurrent(parser, parser->current.start);
  }
}

static void consume(Parser *parser, Scanner *scanner, TokenType type,
                    const char *message) {
  if (parser->current.type == type) {
    advance(parser, scanner);
    return;
  }
  errorAtCurrent(parser, message);
}

// TODO: maybe chunk should be the current chunk
static void emitByte(Parser *parser, Chunk *chunk, uint8_t op) {
  writeChunk(chunk, op, parser->previous.line);
}

static void emitReturn(Parser *parser, Chunk *chunk) {
  emitByte(parser, chunk, OP_RETURN);
}

static void endCompiler(Parser *parser, Chunk *chunk) {
  emitReturn(parser, chunk);
#ifdef DEBUG_PRINT_CODE
  if (!parser->hadError) {
    printf("== Compiled code ==\n");
    disassembleChunk(chunk, "code");
  }
#endif
}

static void emitBytes(Parser *parser, Chunk *chunk, uint8_t op1, uint8_t op2) {
  emitByte(parser, chunk, op1);
  emitByte(parser, chunk, op2);
}

static uint8_t makeConstant(Value value, Parser *parser, Chunk *chunk) {
  size_t constantIndex = addConstant(chunk, value);
  // TODO: we might not need this
  if (constantIndex > UINT8_MAX) {
    error(parser, "Too many constants in one chunk.");
    return 0;
  }
  return (uint8_t)constantIndex;
}

static void emitConstant(Value value, Parser *parser, Chunk *chunk) {
  emitBytes(parser, chunk, OP_CONSTANT, makeConstant(value, parser, chunk));
}

static void number(Parser *parser, Scanner *scanner, Chunk *chunk) {
  double value = strtod(parser->previous.start, NULL);
  emitConstant(value, parser, chunk);
}

typedef void (*ParseFn)(Parser *parser, Scanner *scanner, Chunk *chunk);

typedef struct {
  ParseFn prefix;
  ParseFn infix;
  Precedence precedence;
} ParseRule;

static void expression(Parser *parser, Scanner *scanner, Chunk *chunk);
static ParseRule *getRule(TokenType type);

static void parsePrecedence(Parser *parser, Scanner *scanner, Chunk *chunk,
                            Precedence precedence) {
  advance(parser, scanner);

  ParseFn prefixRule = getRule(parser->previous.type)->prefix;
  if (prefixRule == NULL) {
    error(parser, "Expect expression.");
    return;
  }
  prefixRule(parser, scanner, chunk);

  while (precedence <= getRule(parser->current.type)->precedence) {
    advance(parser, scanner);
    ParseFn infixRule = getRule(parser->previous.type)->infix;
    infixRule(parser, scanner, chunk);
  }
}

static void binary(Parser *parser, Scanner *scanner, Chunk *chunk) {
  TokenType operatorType = parser->previous.type;
  ParseRule *rule = getRule(operatorType);
  parsePrecedence(parser, scanner, chunk, (Precedence)(rule->precedence + 1));

  switch (operatorType) {
  case TOKEN_PLUS:
    emitByte(parser, chunk, OP_ADD);
    break;
  case TOKEN_MINUS:
    emitByte(parser, chunk, OP_SUBTRACT);
    break;
  case TOKEN_STAR:
    emitByte(parser, chunk, OP_MULTIPLY);
    break;
  case TOKEN_SLASH:
    emitByte(parser, chunk, OP_DIVIDE);
    break;
  default:
    return; // Unreachable
  }
}

static void expression(Parser *parser, Scanner *scanner, Chunk *chunk) {
  parsePrecedence(parser, scanner, chunk, PREC_ASSIGNMENT);
}

static void grouping(Parser *parser, Scanner *scanner, Chunk *chunk) {
  expression(parser, scanner, chunk);
  consume(parser, scanner, TOKEN_RIGHT_PAREN, "Expect ')' after expression.");
}

static void unary(Parser *parser, Scanner *scanner, Chunk *chunk) {
  TokenType operatorType = parser->previous.type;
  parsePrecedence(parser, scanner, chunk, PREC_UNARY);
  switch (operatorType) {
  case TOKEN_MINUS:
    emitByte(parser, chunk, OP_NEGATE);
    break;
  // case TOKEN_BANG:
  //   emitByte(parser, chunk, OP_NOT);
  //   break;
  default:
    return; // Unreachable
  }
}

ParseRule rules[] = {[TOKEN_LEFT_PAREN] = {grouping, NULL, PREC_NONE},
                     [TOKEN_RIGHT_PAREN] = {NULL, NULL, PREC_NONE},
                     [TOKEN_LEFT_BRACE] = {NULL, NULL, PREC_NONE},
                     [TOKEN_RIGHT_BRACE] = {NULL, NULL, PREC_NONE},
                     [TOKEN_COMMA] = {NULL, NULL, PREC_NONE},
                     [TOKEN_DOT] = {NULL, NULL, PREC_NONE},
                     [TOKEN_MINUS] = {unary, binary, PREC_TERM},
                     [TOKEN_PLUS] = {NULL, binary, PREC_TERM},
                     [TOKEN_SEMICOLON] = {NULL, NULL, PREC_NONE},
                     [TOKEN_SLASH] = {NULL, binary, PREC_FACTOR},
                     [TOKEN_STAR] = {NULL, binary, PREC_FACTOR},
                     [TOKEN_BANG] = {NULL, NULL, PREC_NONE},
                     [TOKEN_BANG_EQUAL] = {NULL, NULL, PREC_NONE},
                     [TOKEN_EQUAL] = {NULL, NULL, PREC_NONE},
                     [TOKEN_EQUAL_EQUAL] = {NULL, NULL, PREC_NONE},
                     [TOKEN_GREATER] = {NULL, NULL, PREC_NONE},
                     [TOKEN_GREATER_EQUAL] = {NULL, NULL, PREC_NONE},
                     [TOKEN_LESS] = {NULL, NULL, PREC_NONE},
                     [TOKEN_LESS_EQUAL] = {NULL, NULL, PREC_NONE},
                     [TOKEN_IDENTIFIER] = {NULL, NULL, PREC_NONE},
                     [TOKEN_STRING] = {NULL, NULL, PREC_NONE},
                     [TOKEN_NUMBER] = {number, NULL, PREC_NONE},
                     [TOKEN_AND] = {NULL, NULL, PREC_AND},
                     [TOKEN_CLASS] = {NULL, NULL, PREC_NONE},
                     [TOKEN_ELSE] = {NULL, NULL, PREC_NONE},
                     [TOKEN_FALSE] = {NULL, NULL, PREC_NONE},
                     [TOKEN_FOR] = {NULL, NULL, PREC_NONE},
                     [TOKEN_FUN] = {NULL, NULL, PREC_NONE},
                     [TOKEN_IF] = {NULL, NULL, PREC_NONE},
                     [TOKEN_NIL] = {NULL, NULL, PREC_NONE},
                     [TOKEN_OR] = {NULL, NULL, PREC_OR},
                     [TOKEN_PRINT] = {NULL, NULL, PREC_NONE},
                     [TOKEN_RETURN] = {NULL, NULL, PREC_NONE},
                     [TOKEN_SUPER] = {NULL, NULL, PREC_NONE},
                     [TOKEN_THIS] = {NULL, NULL, PREC_NONE},
                     [TOKEN_TRUE] = {NULL, NULL, PREC_NONE},
                     [TOKEN_VAR] = {NULL, NULL, PREC_NONE},
                     [TOKEN_WHILE] = {NULL, NULL, PREC_NONE},
                     [TOKEN_ERROR] = {NULL, NULL, PREC_NONE},
                     [TOKEN_EOF] = {NULL, NULL, PREC_NONE}};

static ParseRule *getRule(TokenType type) { return &rules[type]; }

bool compile(Scanner *scanner, Chunk *chunk, const char *source) {
  Parser parser;
  initScanner(scanner, source);
  parser.hadError = false;
  parser.panicMode = false;
  advance(&parser, scanner);

  expression(&parser, scanner, chunk);
  consume(&parser, scanner, TOKEN_EOF, "Expect end of file.");

  endCompiler(&parser, chunk);
  return !parser.hadError;
}
