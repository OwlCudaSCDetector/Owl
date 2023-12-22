
#ifndef PREFIX
#define PREFIX ""
#endif

#define cBLK "\x1b[0;30m"
#define cRED "\x1b[0;31m"
#define cGRN "\x1b[0;32m"
#define cBRN "\x1b[0;33m"
#define cBLU "\x1b[0;34m"
#define cMGN "\x1b[0;35m"
#define cCYA "\x1b[0;36m"
#define cLGR "\x1b[0;37m"
#define cGRA "\x1b[1;90m"
#define cLRD "\x1b[1;91m"
#define cLGN "\x1b[1;92m"
#define cYEL "\x1b[1;93m"
#define cLBL "\x1b[1;94m"
#define cPIN "\x1b[1;95m"
#define cLCY "\x1b[1;96m"
#define cBRI "\x1b[1;97m"
#define cRST "\x1b[0m"

#define SAYF(...) printf(__VA_ARGS__)

/* Show a prefixed warning. */
#define WARNF(...)                                                             \
  do {                                                                         \
    SAYF(cYEL "[!] " PREFIX cRST);                            \
    SAYF(" - ");                                                               \
    SAYF(__VA_ARGS__);                                                         \
    SAYF(cRST "\n");                                                           \
  } while (0)

/* Show a prefixed "doing something" message. */

#define ACTF(...)                                                              \
  do {                                                                         \
    SAYF(cBLU "[*] " PREFIX cRST);                                             \
    SAYF(" - ");                                                               \
    SAYF(__VA_ARGS__);                                                         \
    SAYF(cRST "\n");                                                           \
  } while (0)

/* Show a prefixed "success" message. */

#define OKF(...)                                                               \
  do {                                                                         \
    SAYF(cGRN "[+] " PREFIX cRST);                                             \
    SAYF(" - ");                                                               \
    SAYF(__VA_ARGS__);                                                         \
    SAYF(cRST "\n");                                                           \
  } while (0)

/* Show a prefixed fatal error message (not used in afl). */

#define BADF(...)                                                              \
  do {                                                                         \
    SAYF(cRED "\n[-] " PREFIX cRST);                                           \
    SAYF(" - ");                                                               \
    SAYF(__VA_ARGS__);                                                         \
    SAYF(cRST "\n");                                                           \
  } while (0)
