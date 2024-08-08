#pragma once

namespace troad::net {
enum class State : int { Connected, Status, Login, Play };

enum class ConnectionPacketTypes : int { Handshake };
enum class StatusPacketTypes : int {
  StatusRequest,
  StatusResponse = StatusRequest
};
}  // namespace troad::net
