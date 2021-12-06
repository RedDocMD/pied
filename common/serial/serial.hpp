#include <iostream>
#include <string>

namespace serial {

class SerialIO {
    std::string tty_name;
    int serial_port;

public:
    static constexpr int baud_rate = 921600;

    SerialIO(const std::string &tty_name);
    ~SerialIO();
    void putc(char c) const;
    char getc() const;
};

}; // namespace serial