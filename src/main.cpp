#include <iostream>

#include "net/server.hpp"

int main(int, char**) {
  try {
    troad::net::Server server(25565);
  } catch (std::exception& e) {
    std::cout << e.what() << std::endl;
  }
}
