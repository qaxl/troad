#pragma once

#include <optional>
#include <span>
#include <string>

namespace troad {
class ByteReader {
 public:
  ByteReader(std::span<uint8_t> data)
      : data_(data.data()), data_len_(data.size()), cursor_(0) {}

  // This function is not recommended to be used with whole classes or similar.
  // It must be a DATA type or a scalar type. Such as: int, struct Vec2 { int x,
  // y; }. Type padding isn't either taken into account, so either pad your
  // structs or use this function for each of it's members.
  template <typename T>
  std::optional<T> Read() {
    if ((data_len_ - cursor_) < sizeof(T)) {
      return std::nullopt;
    }

    T v;
    memcpy(&v, data_ + cursor_, sizeof(T));
    cursor_ += sizeof(T);

    return v;
  }

  template <>
  std::optional<std::string> Read();
  template <>
  std::optional<std::string_view> Read();

 private:
  uint8_t* data_;
  size_t data_len_;
  size_t cursor_;
};

template <typename T>
std::optional<T> ReadVarInteger(ByteReader& reader) {
  T value = T(0);
  uint64_t shift = 0;

  while (true) {
    auto byte = reader.Read<uint8_t>();
    if (!byte) {
      return std::nullopt;
    }

    value |= T(byte.value() & 0x7F) << shift;
    shift += 7;

    if (shift > sizeof(T) * 8) {
      return std::nullopt;
    }

    if ((value & 0x80) == 0) {
      return value;
    }
  }
}

template <>
inline std::optional<std::string> ByteReader::Read<std::string>() {
  auto v = Read<std::string_view>();
  if (!v) {
    return std::nullopt;
  }

  return std::string(*v);
}

template <>
inline std::optional<std::string_view> ByteReader::Read<std::string_view>() {
  auto len = ReadVarInteger<uint32_t>(*this);
  if (!len) {
    return std::nullopt;
  }

  uint32_t length = len.value();
  if (data_len_ - cursor_ < length) {
    return std::nullopt;
  }

  std::string_view v(reinterpret_cast<char*>(data_ + cursor_), length);
  cursor_ += length;

  return v;
}

}  // namespace troad
