#include "vm.h"
#include "compiler.h"
#include "chunk.h"
#include "debug.h"
#include "value.h"
#include <stdio.h>

static void resetStack(VM *vm) { vm->stackTop = vm->stack; }

void initVM(VM *vm) { resetStack(vm); }

void pushVM(VM *vm, Value value) {
  // NOTE: ponter magic. The stacktop points to the location in the array, so
  // when we set the value in puts it in the array
  // then when we increase stacktop, we are move the pointer to the next
  // location
  *vm->stackTop = value;
  vm->stackTop++;
}

Value popVM(VM *vm) {
  vm->stackTop--;
  return *vm->stackTop;
}

static InterpretResult run(VM *vm) {
#define READ_BYTE() (*vm->ip++)
#define READ_CONSTANT() (vm->chunk->constants.values[READ_BYTE()])
#define BINARY_OP(op)                                                          \
  do {                                                                         \
    double b = popVM(vm);                                                      \
    double a = popVM(vm);                                                      \
    pushVM(vm, a op b);                                                        \
  } while (false)

  for (;;) {

#ifdef DEBUG_TRACE_EXECUTION
    printf("    ");
    for (Value *slot = vm->stack; slot < vm->stackTop; slot++) {
      printf("[ ");
      printValue(*slot);
      printf(" ]");
    }
    printf("\n");
    disassembleInstruction(vm->chunk, (int)(vm->ip - vm->chunk->code));
#endif

    uint8_t instruction;
    switch (instruction = READ_BYTE()) {
    case OP_ADD:
      BINARY_OP(+);
      break;
    case OP_MULTIPLY:
      BINARY_OP(*);
      break;
    case OP_SUBTRACT:
      BINARY_OP(-);
      break;
    case OP_DIVIDE:
      BINARY_OP(/);
      break;
    case OP_RETURN: {
      Value popedValue = popVM(vm);
      printValue(popedValue);
      printf("\n");
      return INTERPRET_OK;
    }
    case OP_CONSTANT: {
      Value constant = READ_CONSTANT();
      printf("const \n");
      printValue(constant);
      printf("\n");
      pushVM(vm, constant);
      break;
    }
    case OP_NEGATE: {
      Value popedValue = popVM(vm);
      printf("negate \n");
      printValue(popedValue);
      printf("\n");
      pushVM(vm, -popedValue);
      break;
    }
    }
  }
#undef READ_BYTE
#undef READ_CONSTANT
#undef BINARY_OP
}

InterpretResult interpret(VM* vm, const char *source) {
  printf("interpreter called\n");
  Scanner scanner; //TODO: maybe remove
  Chunk chunk;
  if (!compile(&scanner, &chunk, source)){
    freeChunk(&chunk);
    return INTERPRET_COMPILE_ERROR;
  };
  vm->chunk = &chunk;
  vm->ip = vm->chunk->code;

  InterpretResult result = run(vm);
  freeChunk(&chunk);
  return result;
}

void freeVM(VM *vm) {}
