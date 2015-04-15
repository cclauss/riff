require 'colors'
require 'refiner'
require 'diff_string'

# Call do_stream() with the output of some diff-like tool (diff,
# diff3, git diff, ...) and it will highlight that output for you.
class Riff
  DIFF_HEADER = /^diff /
  DIFF_HUNK_HEADER = /^@@ /

  DIFF_ADDED = /^\+(.*)/
  DIFF_REMOVED = /^-(.*)/
  DIFF_CONTEXT = /^ /

  include Colors

  LINE_PREFIX = {
    initial:          '',
    diff_header:      BOLD,
    diff_hunk_header: CYAN,
    diff_hunk:        '',
    diff_added:       GREEN,
    diff_removed:     RED,
    diff_context:     ''
  }

  def initialize()
    @state = :initial

    @replace_old = ''
    @replace_new = ''
  end

  def handle_initial_line(line)
    if line =~ DIFF_HEADER
      @state = :diff_header
    end
  end

  def handle_diff_header_line(line)
    if line =~ DIFF_HUNK_HEADER
      @state = :diff_hunk_header
    end
  end

  def handle_diff_hunk_header_line(line)
    handle_diff_hunk_line(line)
  end

  def handle_diff_hunk_line(line)
    case line
    when DIFF_HUNK_HEADER
      @state = :diff_hunk_header
    when DIFF_HEADER
      @state = :diff_header
    when DIFF_ADDED
      @state = :diff_added
    when DIFF_REMOVED
      @state = :diff_removed
    when DIFF_CONTEXT
      @state = :diff_context
    end
  end

  def handle_diff_added_line(line)
    handle_diff_hunk_line(line)
  end

  def handle_diff_removed_line(line)
    handle_diff_hunk_line(line)
  end

  def handle_diff_context_line(line)
    handle_diff_hunk_line(line)
  end

  # If we have stored adds / removes, calling this method will flush
  # those.
  def consume_replacement()
    return if @replace_old.empty? && @replace_new.empty?

    refiner = Refiner.new(@replace_old, @replace_new)
    print refiner.refined_old
    print refiner.refined_new

    @replace_old = ''
    @replace_new = ''
  end

  def print_styled_line(style, line)
    reset = (style.empty? ? '' : RESET)
    puts "#{style}#{line}#{reset}"
  end

  # Call handle_<state>_line() for the given state and line
  def handle_line_for_state(state, line)
    method_name = "handle_#{state}_line"
    fail "Unknown state: <:#{state}>" unless
      self.respond_to? method_name

    send(method_name, line)
  end

  def handle_diff_line(line)
    line.chomp!

    handle_line_for_state(@state, line)

    case @state
    when :diff_added
      @replace_new += DIFF_ADDED.match(line)[1] + "\n"
    when :diff_removed
      @replace_old += DIFF_REMOVED.match(line)[1] + "\n"
    else
      consume_replacement()

      color = LINE_PREFIX.fetch(@state)

      print DiffString.decorate_string('', color, line + "\n")
    end
  end

  # Read diff from a stream and output a highlighted version to stdout
  def do_stream(diff_stream)
    diff_stream.each do |line|
      handle_diff_line(line)
    end
    consume_replacement()
  end
end
