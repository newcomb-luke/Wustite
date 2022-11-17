#include "ctype.h"

bool islower(char c) {
    return c >= 'a' && c <= 'z';
}

bool isupper(char c) {
    return c >= 'A' && c <= 'z';
}

char tolower(char c) {
    char result = c;

    if (isupper(c)) {
        result += 'a' - 'A';
    }

    return result;
}

char toupper(char c) {
    char result = c;

    if (islower(c)) {
        result -= 'a' - 'A';
    }

    return result;
}
