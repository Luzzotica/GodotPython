import sys

class CustomStream:
    def __init__(self, original_stream, callback):
        self.original_stream = original_stream
        self.callback = callback

    def write(self, message):
        self.original_stream.write(message)
        self.callback(message)

    def flush(self):
        self.original_stream.flush()

def my_callback(message):
    print(f"Custom function called with message: {message}")

# Save the original stdout
original_stdout = sys.stdout

# Set the custom stream as the new stdout
sys.stdout = CustomStream(original_stdout, my_callback)

# Test the custom stdout
print("Hello, World!")

# Restore the original stdout
sys.stdout = original_stdout

print("Back to normal stdout")