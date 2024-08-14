import sys
from rust_stdout import rs_print, rs_print_err

class CustomStream:

    def __init__(self, original_stream, callback, start="", end=""):
        self.original_stream = original_stream
        self.callback = callback
        self.buffer = start
        self.start = start
        self.end = end

    def write(self, message):
        self.buffer += message
        if "\n" == message:
            self.buffer += self.end
            self.flush()

    def flush(self):
        if self.buffer:
            self.original_stream.write(self.buffer)
            self.callback(self.buffer)
            self.buffer = self.start


std = sys.stdout
err = sys.stderr


def set_rust_stdout(rstd):
    # Set the custom stream as the new stdout
    sys.stdout = CustomStream(std, rstd.rs_print)
    sys.stderr = CustomStream(
        err, rstd.rs_print_err, start="[color=red]", end="[/color]"
    )
    # print("swag from python")
    # print("swag from python 2")


sys.stdout = CustomStream(std, rs_print)
sys.stderr = CustomStream(err, rs_print_err, start="[color=red]", end="[/color]")
