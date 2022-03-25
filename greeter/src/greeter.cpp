#include <cstdio>
#include <cstdint>

extern "C" {
// Provided by the Rust library
void greet(uint32_t excitedness);
}

int main(int argc, char ** argv)
{
  (void) argc;
  (void) argv;

  greet(4);
  return 0;
}
