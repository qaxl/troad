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
  DoAcceptConnection();

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

void Server::DoAcceptConnection() {
  acceptor_.async_accept(
      [this](std::error_code ec, asio::ip::tcp::socket socket) {
        if (!ec) {
          std::lock_guard<std::mutex> lock(sessions_lock_);
          sessions_.emplace_back(
              std::make_shared<Session>(std::move(socket), *this));
        }

        DoAcceptConnection();
      });
}

}  // namespace troad::net
