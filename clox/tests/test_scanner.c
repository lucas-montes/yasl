// clox/tests/test_scanner.c
#include "../scanner.h"
#include <assert.h>
#include <stdio.h>

void test_scanner_plus() {
    Scanner scanner;
    initScanner(&scanner, "+");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_PLUS);
    assert(token.length == 1);
    assert(scanner.line == 1);
}

void test_scanner_whitespace() {
    Scanner scanner;
    initScanner(&scanner, "   +   ");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_PLUS);
    assert(token.length == 1);
    assert(scanner.line == 1);
}
void test_scanner_comments() {
    Scanner scanner;
    initScanner(&scanner, "// This is a comment\n+");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_PLUS);
    assert(token.length == 1);
    assert(scanner.line == 2);
}
void test_scanner_multiline_comments() {
    Scanner scanner;
    initScanner(&scanner, "/* This is a\nmultiline comment */ +");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_PLUS);
    assert(token.length == 1);
    assert(scanner.line == 2);
}

void test_scanner_string() {
    Scanner scanner;
    initScanner(&scanner, "\"Hello, World!\"");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_STRING);
    assert(token.length == 15); //it have to include the quotes
    assert(scanner.line == 1);
}

void test_scanner_string_unterminated() {
    Scanner scanner;
    initScanner(&scanner, "\"Hello, World!");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_ERROR);
}

void test_this() {
    Scanner scanner;
    initScanner(&scanner, "this");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_THIS);
    assert(token.length == 4);
    assert(scanner.line == 1);
}

void test_else() {
    Scanner scanner;
    initScanner(&scanner, "else");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_ELSE);
    assert(token.length == 4);
    assert(scanner.line == 1);
}
void test_scanner_identifier() {
    Scanner scanner;
    initScanner(&scanner, "myVariable");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_IDENTIFIER);
    assert(token.length == 10);
    assert(scanner.line == 1);
}
void test_scanner_number() {
    Scanner scanner;
    initScanner(&scanner, "12345");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_NUMBER);
    assert(token.length == 5);
    assert(scanner.line == 1);
}
void test_scanner_number_with_decimal() {
    Scanner scanner;
    initScanner(&scanner, "123.45");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_NUMBER);
    assert(token.length == 6);
    assert(scanner.line == 1);
}
void test_scanner_number_with_leading_zero() {
    Scanner scanner;
    initScanner(&scanner, "0123");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_NUMBER);
    assert(token.length == 4);
    assert(scanner.line == 1);
}
void test_scanner_number_with_leading_zero_decimal() {
    Scanner scanner;
    initScanner(&scanner, "0.123");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_NUMBER);
    assert(token.length == 5);
    assert(scanner.line == 1);
}

void test_for() {
    Scanner scanner;
    initScanner(&scanner, "for");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_FOR);
    assert(token.length == 3);
    assert(scanner.line == 1);
}
void test_if() {
    Scanner scanner;
    initScanner(&scanner, "if");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_IF);
    assert(token.length == 2);
    assert(scanner.line == 1);
}
void test_and() {
    Scanner scanner;
    initScanner(&scanner, "and");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_AND);
    assert(token.length == 3);
    assert(scanner.line == 1);
}
void test_or() {
    Scanner scanner;
    initScanner(&scanner, "or");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_OR);
    assert(token.length == 2);
    assert(scanner.line == 1);
}
void test_nill() {
    Scanner scanner;
    initScanner(&scanner, "nil");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_NIL);
    assert(token.length == 3);
    assert(scanner.line == 1);
}
void test_class() {
    Scanner scanner;
    initScanner(&scanner, "class");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_CLASS);
    assert(token.length == 5);
    assert(scanner.line == 1);
}
void test_return() {
    Scanner scanner;
    initScanner(&scanner, "return");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_RETURN);
    assert(token.length == 6);
    assert(scanner.line == 1);
}
void test_print() {
    Scanner scanner;
    initScanner(&scanner, "print");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_PRINT);
    assert(token.length == 5);
    assert(scanner.line == 1);
}
void test_super() {
    Scanner scanner;
    initScanner(&scanner, "super");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_SUPER);
    assert(token.length == 5);
    assert(scanner.line == 1);
}
void test_true() {
    Scanner scanner;
    initScanner(&scanner, "true");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_TRUE);
    assert(token.length == 4);
    assert(scanner.line == 1);
}
void test_false() {
    Scanner scanner;
    initScanner(&scanner, "false");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_FALSE);
    assert(token.length == 5);
    assert(scanner.line == 1);
}
void test_var() {
    Scanner scanner;
    initScanner(&scanner, "var");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_VAR);
    assert(token.length == 3);
    assert(scanner.line == 1);
}
void test_while() {
    Scanner scanner;
    initScanner(&scanner, "while");
    Token token = scanToken(&scanner);
    assert(token.type == TOKEN_WHILE);
    assert(token.length == 5);
    assert(scanner.line == 1);
}

int main() {
    test_scanner_plus();
    test_scanner_whitespace();
    test_scanner_comments();
    test_scanner_multiline_comments();
    test_scanner_string();
    test_scanner_string_unterminated();
    test_this();
    test_else();
    test_scanner_identifier();
    test_scanner_number();
    test_scanner_number_with_decimal();
    test_scanner_number_with_leading_zero();
    test_scanner_number_with_leading_zero_decimal();
    test_for();
    test_if();
    test_and();
    test_or();
    test_nill();
    test_class();
    test_return();
    test_print();
    test_super();
    test_true();
    test_false();
    test_var();
    test_while();
    printf("✅ Scanner tests passed.\n");
    return 0;
}
