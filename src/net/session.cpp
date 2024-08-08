#include "session.hpp"

#include <asio.hpp>
#include <asio/buffer.hpp>
#include <iostream>

#include "state.hpp"
#include "util/binary.hpp"

namespace troad::net {

void Session::DoHandleConnection() {
  socket_.async_read_some(
      asio::buffer(buf_), [this](std::error_code ec, size_t read) {
        if (!ec) {
          ByteReader reader(std::span(buf_, read));
          auto whole_size = ReadVarInteger<uint32_t>(reader);
          if (whole_size.has_value()) {
            std::cout << "Receiving " << whole_size.value()
                      << " bytes from client." << std::endl;

            uint32_t whole = whole_size.value();
            if (whole > read) {
              std::cout << "unhandled scenario, where whole is less than read."
                        << std::endl;
              // Temporarily, just handle this...
              DoHandleByteStream(reader);
              DoHandleConnection();
            } else if (whole > read) {
              std::cout << "unhandled scenario, where whole is more than read."
                        << std::endl;
            } else {
              DoHandleByteStream(reader);
              DoHandleConnection();
            }
          }
        } else {
          DoHandleConnection();
        }
      });
}

// FIXME: temporary
struct JsonMessage {
  StatusPacketTypes type;
};

void Session::DoHandleByteStream(ByteReader reader) {
  uint32_t type = ReadVarInteger<uint32_t>(reader).value_or(-1);

  switch (state_) {
    case State::Connected:
      switch (static_cast<ConnectionPacketTypes>(type)) {
        case ConnectionPacketTypes::Handshake:
          uint32_t protocol_version =
              ReadVarInteger<uint32_t>(reader).value_or(-1);
          std::string address = reader.Read<std::string>().value_or("");
          uint16_t port = reader.Read<uint16_t>().value_or(0);
          State next_state =
              static_cast<State>(ReadVarInteger<uint32_t>(reader).value_or(0));

          std::cout << "Receiving handshake: next_state = "
                    << static_cast<int>(next_state)
                    << ", protocol_version = " << protocol_version
                    << ", address:port = " << address << ':' << port
                    << std::endl;
          if (next_state != State::Connected && next_state != State::Play) {
            state_ = next_state;
          }
      }
      break;

    case State::Status:
      switch (static_cast<StatusPacketTypes>(type)) {
        case StatusPacketTypes::StatusRequest:

          break;
      }
      break;
    default:
      std::cout << "Received unhandled state type of "
                << static_cast<int>(state_) << std::endl;
      break;
  }
}

}  // namespace troad::net
