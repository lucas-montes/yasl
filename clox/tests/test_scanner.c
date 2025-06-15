// clox/tests/test_scanner.c
#include "../scanner.h"
#include <assert.h>
#include <stdio.h>

void test_scanner_plus() {
    Scanner scanner;
    initScanner(&scanner, "+");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_PLUS);
    assert(token.lenght == 1);
    assert(scanner.line == 1);
}

void test_scanner_whitespace() {
    Scanner scanner;
    initScanner(&scanner, "   +   ");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_PLUS);
    assert(token.lenght == 1);
    assert(scanner.line == 1);
}
void test_scanner_comments() {
    Scanner scanner;
    initScanner(&scanner, "// This is a comment\n+");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_PLUS);
    assert(token.lenght == 1);
    assert(scanner.line == 2);
}
void test_scanner_multiline_comments() {
    Scanner scanner;
    initScanner(&scanner, "/* This is a\nmultiline comment */ +");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_PLUS);
    assert(token.lenght == 1);
    assert(scanner.line == 2);
}

void test_scanner_string() {
    Scanner scanner;
    initScanner(&scanner, "\"Hello, World!\"");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_STRING);
    assert(token.lenght == 15); //it have to include the quotes
    assert(scanner.line == 1);
}


int main() {
    test_scanner_plus();
    test_scanner_whitespace();
    test_scanner_comments();
    test_scanner_multiline_comments();
    test_scanner_string();
    printf("✅ Scanner tests passed.\n");
    return 0;
}
