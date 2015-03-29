# Call do_stream() with the output of some diff-like tool (diff,
# diff3, git diff, ...) and it will highlight that output for you.
class Riff
  DIFF_HEADER = /^diff /
  DIFF_HUNK_HEADER = /^@@ /

  DIFF_ADDED = /^\+/
  DIFF_REMOVED = /^-/
  DIFF_CONTEXT = /^ /

  ESC = 27.chr
  R = "#{ESC}[7m"  # REVERSE
  N = "#{ESC}[27m" # NORMAL

  BOLD = "#{ESC}[1m"
  CYAN = "#{ESC}[36m"
  GREEN = "#{ESC}[32m"
  RED = "#{ESC}[31m"

  RESET = "#{ESC}[m"

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

  def handle_diff_line(line)
    line.chomp!

    method_name = "handle_#{@state}_line"
    fail "Unknown state: <:#{@state}>" unless
      self.respond_to? method_name

    send(method_name, line)

    style = LINE_PREFIX.fetch(@state)
    reset = (style.empty? ? '' : RESET)
    puts "#{style}#{line}#{reset}"
  end

  # Read diff from a stream and output a highlighted version to stdout
  def do_stream(diff_stream)
    diff_stream.each do |line|
      handle_diff_line(line)
    end
  end
end
