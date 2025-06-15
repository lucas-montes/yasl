#include "compiler.h"
#include "scanner.h"
#include <stdio.h>
#include <stdlib.h>

typedef struct {
  Token current;
  Token previous;
  bool hadError;
  bool panicMode;
} Parser;

static void errorAt(Parser* parser, Token *token, const char *message) {
  if (parser->panicMode) return;
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

static void advance(Parser *parser) {
  parser->previous = parser->current;
  for (;;){
    parser->current = scanToken();
    if (parser->current.type != TOKEN_ERROR) break;

    errorAtCurrent(parser, parser->current.start);
  }
}

static void consume(Parser *parser, TokenType type, const char *message) {
  if (parser->current.type == type) {
    advance(parser);
    return;
  }
  errorAtCurrent(parser, message);
}

  // TODO: global variable that holds the current chunk, to delete
Chunk* compilingChunk;

static Chunk* currentChunk() {
  return compilingChunk;
}

//TODO: maybe chunk should be the current chunk
static void emitByte(Parser* parser, Chunk *chunk, uint8_t byte) {
  writeChunk(currentChunk(), byte, parser->previous.line);
}

static void endCompiler(Parser *parser, Chunk *chunk) {
  emitByte(parser, chunk, OP_RETURN);
}

static void emitBytes(Parser *parser, Chunk *chunk, uint8_t byte1, uint8_t byte2) {
  emitByte(parser, chunk, byte1);
  emitByte(parser, chunk, byte2);
}

static uint8_t makeConstant(Value value, Parser *parser) {
  size_t constantIndex = addConstant(currentChunk(), value);
  if (constantIndex > UINT8_MAX) {
    error(parser, "Too many constants in one chunk.");
    return 0;
  }
  return (uint8_t)constantIndex;
}

static void emitConstant(Value value, Parser *parser, Chunk *chunk) {
  emitBytes(parser, chunk, OP_CONSTANT, makeConstant(value, parser));
}

static void number(Parser *parser){
  double value = strtod(parser->previous.start, NULL);
  emitConstant(value);
}

static void expression() {

}

bool compile(Scanner *scanner, Chunk* chunk, const char *source) {
  Parser parser;
  initScanner(scanner, source);
  compilingChunk = chunk; //TODO: ugly
  parser.hadError = false;
  parser.panicMode = false;
  advance(&parser);
  expression();
  consume(TOKEN_EOF, "Expect end of file.");
  endCompiler(&parser, chunk);
  return !parser.hadError;
}
