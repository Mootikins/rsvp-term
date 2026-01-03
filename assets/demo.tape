# VHS Demo for rsvp-term
# https://github.com/charmbracelet/vhs

Output demo.gif

Set FontSize 16
Set Width 1000
Set Height 600
Set Theme "Catppuccin Mocha"
Set TypingSpeed 50ms

# Show the command
Type "rsvp-term assets/alice-chapter2.md"
Sleep 500ms
Enter
Sleep 2s

# Let it run - reading the intro
Sleep 4s

# Pause to show the interface
Type " "
Sleep 1.5s

# Speed up (press k four times to 400 WPM)
Type "kkkk"
Sleep 1s

# Resume at faster speed
Type " "
Sleep 4s

# Show help overlay
Type "?"
Sleep 2.5s

# Close help
Type "?"
Sleep 1s

# Let it read more
Sleep 3s

# Quit
Type "q"
Sleep 500ms
