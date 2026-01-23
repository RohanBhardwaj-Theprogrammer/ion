#include "util.h"
#include "logger.h"
int add(int a, int b) {
    log_message("add() called");
    return a + b;
}
