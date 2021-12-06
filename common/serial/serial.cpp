#include "serial.hpp"
#include <cstring>
#include <errno.h>
#include <fcntl.h>
#include <iostream>
#include <termios.h>
#include <unistd.h>

namespace serial {

SerialIO::SerialIO(const std::string &tty_name) : tty_name(tty_name) {
    serial_port = open(tty_name.c_str(), O_RDWR);

    if (serial_port < 0) {
        std::cerr << "Failed to open " << tty_name << ": " << strerror(errno)
                  << std::endl;
        std::exit(1);
    }

    termios tty;

    if (tcgetattr(serial_port, &tty) != 0) {
        std::cerr << "Failed to tcgetattr: " << strerror(errno) << std::endl;
        std::exit(1);
    }

    // Set cflag
    tty.c_cflag &= ~PARENB;
    tty.c_cflag &= ~CSTOPB;
    tty.c_cflag |= CS8;
    tty.c_cflag &= ~CRTSCTS;
    tty.c_cflag |= (CREAD | CLOCAL);

    // Set lflag
    tty.c_lflag &= ~ICANON;
    tty.c_lflag &= ~ECHO;
    tty.c_lflag &= ~ECHOE;
    tty.c_lflag &= ~ECHONL;
    tty.c_lflag &= ~ISIG;

    // Set iflag
    tty.c_iflag &= ~(IXON | IXOFF | IXANY);
    tty.c_iflag &= ~(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL);

    // Set oflag
    tty.c_oflag &= ~OPOST;
    tty.c_oflag &= ~ONLCR;

    // Set timeouts
    tty.c_cc[VMIN] = 1;
    tty.c_cc[VTIME] = 0;

    // Set baud-rate
    cfsetspeed(&tty, SerialIO::baud_rate);

    // Save termios data
    if (tcsetattr(serial_port, TCSANOW, &tty) != 0) {
        std::cerr << "Failed to tcsetattr:  " << strerror(errno) << std::endl;
        std::exit(1);
    }
}

SerialIO::~SerialIO() { close(serial_port); }

void SerialIO::putc(char c) const {
    int n = write(serial_port, &c, 1);
    if (n != 1) {
        std::cerr << "Failed to write character to " << tty_name << std::endl;
        std::exit(1);
    }
}

char SerialIO::getc() const {
    char ch;
    int n = read(serial_port, &ch, 1);
    if (n != 1) {
        std::cerr << "Failed to read character from " << tty_name << std::endl;
        std::exit(1);
    }
    return ch;
}

} // namespace serial