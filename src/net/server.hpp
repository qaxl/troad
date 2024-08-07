#pragma once

#include <asio.hpp>
#include <asio/io_context.hpp>
#include <vector>

namespace troad::net {
class Server {
 public:
  Server(int port);
  ~Server();

 private:
  void do_accept();

  asio::io_context io_context_;
  asio::ip::tcp::acceptor acceptor_;
  std::vector<std::thread> threads_;
};
}  // namespace troad::net
