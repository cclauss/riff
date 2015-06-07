# coding: utf-8
require 'colors'

# An added or removed part of a diff, which can contain both
# highlighted and not highlighted characters. For multi line strings,
# each line will be prefixed with prefix and color.
class DiffString
  include Colors

  # Note that the color argument can be the empty string
  def initialize(prefix, color)
    @reverse = false
    @prefix = prefix
    @color = color
    @string = ''
  end

  def add(string, reverse)
    if reverse && string == "\n"
      add('↵', true)
      add("\n", false)
      return
    end

    if @string.empty?() || @string.end_with?("\n")
      @string += @color
      @string += @prefix
    end

    if reverse != @reverse
      @string += reverse ? REVERSE : NOT_REVERSE
    end
    @reverse = reverse

    @string += string
  end

  def to_s()
    return '' if @string.empty?

    string = @string
    string.chomp! if string.end_with? "\n"

    suffix = @color.empty? ? '' : RESET
    return string + suffix + "\n"
  end

  def self.decorate_string(prefix, color, string)
    decorated = DiffString.new(prefix, color)
    decorated.add(string, false)
    return decorated.to_s
  end
end
