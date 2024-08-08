#pragma once

#include <asio.hpp>

#include "state.hpp"
#include "util/binary.hpp"

namespace troad::net {
class Server;

class Session {
 public:
  Session(asio::ip::tcp::socket&& socket, Server& server)
      : socket_(std::move(socket)), server_(server) {
    DoHandleConnection();
  }

 private:
  void DoHandleConnection();
  void DoHandleByteStream(ByteReader reader);

  // Initial buffer for received data, if a client sends larger packet - it will
  // be handled specifically.
  uint8_t buf_[2048];
  // The connected socket for this "session"
  asio::ip::tcp::socket socket_;
  // Ref to server
  Server& server_;
  // Session's current status
  State state_ = State::Connected;
};
}  // namespace troad::net
