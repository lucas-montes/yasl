#ifndef clox_compiler_h
#define clox_compiler_h

#include "scanner.h"
#include "chunk.h"

bool compile(Scanner* scanner, Chunk* chunk, const char *source);

#endif
