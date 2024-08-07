# Add your third_party libraries here... 
# Add the path to standalone Asio
set(ASIO_PATH ${CMAKE_CURRENT_SOURCE_DIR}/asio/include)

# Find the Asio headers
find_path(ASIO_INCLUDE_DIR NAMES asio.hpp PATHS ${ASIO_PATH})
