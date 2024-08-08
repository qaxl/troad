#include "server.hpp"

#include <asio/io_context.hpp>
#include <asio/ip/tcp.hpp>
#include <thread>

#include "session.hpp"

namespace troad::net {
Server::Server(int port)
    : io_context_(std::thread::hardware_concurrency()),
      acceptor_(io_context_,
                asio::ip::tcp::endpoint(asio::ip::tcp::v4(), port)) {
  do_accept();

  for (int i = 0; i < std::thread::hardware_concurrency() - 1; ++i) {
    auto& thr = threads_.emplace_back([this]() { io_context_.run(); });
  }

  io_context_.run();
}

Server::~Server() {
  for (auto& thread : threads_) {
    thread.join();
  }
}

void Server::do_accept() {
  acceptor_.async_accept(
      [this](std::error_code ec, asio::ip::tcp::socket socket) {
        if (!ec) {
          std::make_shared<Session>(std::move(socket))->do_handle();
        }

        do_accept();
      });
}

}  // namespace troad::net
