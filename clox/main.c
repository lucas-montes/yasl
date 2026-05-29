#include "vm.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char* readFile(const char* path){
  FILE *file = fopen(path, "rb");
  if (file == NULL) {
    fprintf(stderr, "Could not open file %s.\n", path);
    exit(74);
  }

  fseek(file, 0L, SEEK_END);
  size_t fileSize = ftell(file);
  rewind(file);

  char *buffer = (char*)malloc(fileSize + 1);
  if (buffer == NULL) {
    fprintf(stderr, "Memory allocation failed.\n");
    exit(74); // Out of memory
  }

  size_t bytesRead = fread(buffer, sizeof(char), fileSize, file);
  if (bytesRead < fileSize) {
    fprintf(stderr, "Could not read file %s completely.\n", path);
    exit(74);
  }
  buffer[fileSize] = '\0'; // Null-terminate the string
  fclose(file);

  return buffer;
}

static void runfile(const char *path, VM *vm) {
  char* source = readFile(path);
  InterpretResult result = interpret(vm, source);
  free(source);

  if (result == INTERPRET_COMPILE_ERROR) {
    exit(65); // Compilation error
  } else if (result == INTERPRET_RUNTIME_ERROR) {
    exit(70); // Runtime error
  }
}

static void repl(VM *vm) {
  char line[1024];
  for (;;){
    printf("clox> ");
    if (!fgets(line, sizeof(line), stdin)) {
      printf("\n");
      break; // EOF or error
    }
    if (line[0] == '\n') continue; // skip empty lines

    InterpretResult result = interpret(vm, line);
    if (result == INTERPRET_COMPILE_ERROR) {
      printf("Compile error.\n");
    } else if (result == INTERPRET_RUNTIME_ERROR) {
      printf("Runtime error.\n");
    }
  }
}

int main(int argc, const char *argv[]) {
  VM vm;
  initVM(&vm);

  if (argc == 1) {
    repl(&vm);
  } else if (argc == 2) {
    runfile(argv[1], &vm);
  } else {
    fprintf(stderr, "Usage: clox [path]\n");
    exit(64);
  }
  freeVM(&vm);
  return 0;
}
