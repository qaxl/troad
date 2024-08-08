#pragma once

#include <asio.hpp>
#include <asio/io_context.hpp>
#include <vector>

namespace troad::net {
class Session;

class Server {
 public:
  Server(int port);
  ~Server();

 private:
  void DoAcceptConnection();

  asio::io_context io_context_;
  asio::ip::tcp::acceptor acceptor_;
  std::vector<std::thread> threads_;

  // TODO: do some kind of pooling for different threads/chunks?
  std::mutex sessions_lock_;
  std::vector<std::shared_ptr<Session>> sessions_;
};
}  // namespace troad::net
