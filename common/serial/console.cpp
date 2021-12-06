#include "console.hpp"
#include <cstdlib>
#include <iostream>

namespace console {

char Console::getc() {
    char ch;
    std::cin.get(ch);
    if (!std::cin.good()) {
        std::cerr << "Failed to receive character in Console" << std::endl;
        std::exit(1);
    }
    return ch;
}

void Console::putc(char c) {
    std::cout.put(c);
    std::cout.flush();
    if (!std::cout.good()) {
        std::cerr << "Failed to send character in Console" << std::endl;
        std::exit(1);
    }
}

}; // namespace console