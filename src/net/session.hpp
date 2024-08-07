#pragma once

#include <asio/ip/tcp.hpp>
#include <iostream>
#include <memory>

#include "asio/buffer.hpp"
#include "asio/error_code.hpp"

int read_var_int(char** buf) {
  int value = 0;
  int shift = 0;

  // TODO: handle buffer overflows
  uint8_t byte = 0;
  do {
    byte = (*buf)[shift++];
    value |= static_cast<int>(byte & 0x7F) << shift;
    shift += 7;
  } while ((byte & 0x80) != 0);

  *buf += shift / 7;
  return value;
}

namespace troad::net {
class Session : public std::enable_shared_from_this<Session> {
 public:
  Session(asio::ip::tcp::socket&& socket) : socket_(std::move(socket)) {}

 private:
  friend class Server;

  void do_data() {
    // TODO: abstraction.
    char* buf = initial_buf_;
    int len = read_var_int(&buf);

    std::cout << "Receiving a packet of length: " << len << std::endl;
  }

  void do_handle() {
    // idk... this is necessary ig
    auto self(shared_from_this());

    socket_.async_read_some(
        asio::buffer(initial_buf_),
        [this, self](std::error_code ec, std::size_t length) {
          asio::error_code e;
          if (!ec) {
            do_data();
            do_handle();
          } else {
            std::cout << "Disconnecting socket because of error: " << ec << ", "
                      << ec.message() << ", " << ec.value() << std::endl;
          }
        });
  }

  char initial_buf_[2048];
  asio::ip::tcp::socket socket_;
};
}  // namespace troad::net
