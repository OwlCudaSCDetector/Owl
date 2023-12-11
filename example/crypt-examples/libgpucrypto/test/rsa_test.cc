/* This file includes test utilities for BN functions on device */
#include <cassert>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sys/time.h>
#include <unistd.h>

#include <typeinfo>

#include "common.hh"
#include "rsa_context.hh"
#include "rsa_context_mp.hh"
#include "rsa_context_rns.hh"

#define MAX_BATCH 1
#define MSG_LEN 1024
static unsigned char ptext[MAX_BATCH][MSG_LEN];
static int ptext_len[MAX_BATCH];
static unsigned char ctext[MAX_BATCH][MSG_LEN];
static int ctext_len[MAX_BATCH];
static unsigned char dtext[MAX_BATCH][MSG_LEN];
static int dtext_len[MAX_BATCH];

static void test_correctness(rsa_context *rsa, int iteration) {
  int max_len = rsa->max_ptext_bytes();

  /* step 1: no batch, static text */
  printf("correctness check (no batch): ");
  fflush(stdout);

  strcpy((char *)ptext[0], "hello world, hello RSA");
  ptext_len[0] = strlen((char *)ptext[0]) + 1;
  ctext_len[0] = sizeof(ctext[0]);
  dtext_len[0] = sizeof(dtext[0]);

  rsa->pub_encrypt(ctext[0], &ctext_len[0], ptext[0], ptext_len[0]);
  rsa->priv_decrypt(dtext[0], &dtext_len[0], ctext[0], ctext_len[0]);

  assert(dtext_len[0] == ptext_len[0]);
  assert(strcmp((char *)dtext[0], (char *)ptext[0]) == 0);

  printf("OK\n");

  /* step 2: no batch, random */
  printf("correctness check (no batch, random, iterative): ");
  fflush(stdout);

  for (int k = 0; k < iteration; k++) {
    ptext_len[0] = (rand() % max_len) + 1;
    ctext_len[0] = sizeof(ctext[0]);
    dtext_len[0] = sizeof(dtext[0]);
    set_random(ptext[0], ptext_len[0]);

    rsa->pub_encrypt(ctext[0], &ctext_len[0], ptext[0], ptext_len[0]);
    rsa->priv_decrypt(dtext[0], &dtext_len[0], ctext[0], ctext_len[0]);

    assert(dtext_len[0] == ptext_len[0]);
    assert(strcmp((char *)dtext[0], (char *)ptext[0]) == 0);
    printf(".");
    fflush(stdout);
  }
  printf("OK\n");

  /* step 3: batch, random */
  printf("correctness check (batch, random): ");
  fflush(stdout);

  bool all_correct = true;
  for (int k = 1; k <= MAX_BATCH; k *= 2) {
    unsigned char *ctext_arr[MAX_BATCH];
    unsigned char *dtext_arr[MAX_BATCH];

    for (int i = 0; i < k; i++) {
      ptext_len[i] = (rand() % max_len) + 1;
      ctext_len[i] = sizeof(ctext[i]);
      dtext_len[i] = sizeof(dtext[i]);
      set_random(ptext[i], ptext_len[i]);

      rsa->pub_encrypt(ctext[i], &ctext_len[i], ptext[i], ptext_len[i]);

      dtext_arr[i] = dtext[i];
      ctext_arr[i] = ctext[i];
    }

    rsa->priv_decrypt_batch((unsigned char **)dtext_arr, dtext_len,
                            (const unsigned char **)ctext_arr, ctext_len, k);

    bool correct = true;
    for (int i = 0; i < k; i++) {
      if (dtext_len[i] != ptext_len[i] ||
          memcmp(dtext[i], ptext[i], dtext_len[i]) != 0) {
        correct = false;
      }
    }

    if (correct) {
      printf(".");
    } else {
      printf("X");
      all_correct = false;
    }

    fflush(stdout);
  }

  assert(all_correct);
  printf("OK\n");
}

static void test_latency(rsa_context *rsa) {
  bool warmed_up = false;
  int max_len = rsa->max_ptext_bytes();

  printf("# msg	latency	CPU	kernel	throughput(RSA msgs/s)\n");

  for (int k = 1; k <= MAX_BATCH; k *= 2) {
    // if (k == 32)
    //	k = 30; 	// GTX285 has 30 SMs :)

    unsigned char *ctext_arr[MAX_BATCH];
    unsigned char *dtext_arr[MAX_BATCH];

    uint64_t begin;
    uint64_t end;

    for (int i = 0; i < k; i++) {
      ptext_len[i] = (rand() % max_len) + 1;
      ctext_len[i] = sizeof(ctext[i]);
      dtext_len[i] = sizeof(dtext[i]);
      set_random(ptext[i], ptext_len[i]);

      rsa->pub_encrypt(ctext[i], &ctext_len[i], ptext[i], ptext_len[i]);

      dtext_arr[i] = dtext[i];
      ctext_arr[i] = ctext[i];
    }

  again:
    int iteration = 1;
    begin = get_usec();
  try_more:
    rsa->priv_decrypt_batch((unsigned char **)dtext_arr, dtext_len,
                            (const unsigned char **)ctext_arr, ctext_len, k);

    end = get_usec();
    if (end - begin < 300000) {
      for (int i = 0; i < k; i++)
        dtext_len[i] = sizeof(dtext[i]);
      iteration++;

      if (!warmed_up) {
        warmed_up = true;
        goto again;
      } else
        goto try_more;
    }

    double total_time = (end - begin) / (iteration * 1000.0);
    double kernel_time = rsa->get_elapsed_ms_kernel();
    double throughput = (k * 1000000.0) * iteration / (end - begin);
    printf("%4d\t%.2f\t%.2f\t%.2f\t%.2f\n", k, total_time,
           total_time - kernel_time, kernel_time, throughput);
  }
}

unsigned char ptext_str[rsa_context_mp::max_stream + 1][MAX_BATCH][MSG_LEN];
int ptext_len_str[rsa_context_mp::max_stream + 1][MAX_BATCH];
unsigned char ctext_str[rsa_context_mp::max_stream + 1][MAX_BATCH][MSG_LEN];
int ctext_len_str[rsa_context_mp::max_stream + 1][MAX_BATCH];
unsigned char dtext_str[rsa_context_mp::max_stream + 1][MAX_BATCH][MSG_LEN];
int dtext_len_str[rsa_context_mp::max_stream + 1][MAX_BATCH];
unsigned char *ctext_arr_str[rsa_context_mp::max_stream + 1][MAX_BATCH];
unsigned char *dtext_arr_str[rsa_context_mp::max_stream + 1][MAX_BATCH];

static void test_latency_stream(rsa_context_mp *rsa, device_context *dev_ctx,
                                int concurrency) {
  int max_len = rsa->max_ptext_bytes();

  printf("# msg	throughput(RSA msgs/s)\n");

  for (int k = 1; k <= MAX_BATCH; k *= 2) {
    // if (k == 32)
    //	k = 30; 	// GTX285 has 30 SMs :)

    uint64_t begin;
    uint64_t end;

    for (int s = 1; s <= concurrency; s++) {
      for (int i = 0; i < k; i++) {
        ptext_len_str[s][i] = (rand() % max_len + 1);
        ctext_len_str[s][i] = sizeof(ctext_str[s][i]);
        dtext_len_str[s][i] = sizeof(dtext_str[s][i]);
        set_random(ptext_str[s][i], ptext_len_str[s][i]);

        rsa->pub_encrypt(ctext_str[s][i], &ctext_len_str[s][i], ptext_str[s][i],
                         ptext_len_str[s][i]);

        dtext_arr_str[s][i] = dtext_str[s][i];
        ctext_arr_str[s][i] = ctext_str[s][i];
      }
    }

    // warmup
    for (int i = 1; i < concurrency; i++) {
      rsa->priv_decrypt_stream(
          (unsigned char **)dtext_arr_str[i], dtext_len_str[i],
          (const unsigned char **)ctext_arr_str[i], ctext_len_str[i], k, i);
      rsa->sync(i, true);
    }

    begin = get_usec();
    int rounds = 200;
    int count = 0;
    do {
      int stream = 0;
      for (int i = 1; i <= concurrency; i++) {
        if (dev_ctx->get_state(i) == READY) {
          stream = i;
          break;
        } else {
          if (rsa->sync(i, false)) {
            count++;
            if (count == concurrency)
              begin = get_usec();
          }
        }
      }
      if (stream != 0) {
        rsa->priv_decrypt_stream((unsigned char **)dtext_arr_str[stream],
                                 dtext_len_str[stream],
                                 (const unsigned char **)ctext_arr_str[stream],
                                 ctext_len_str[stream], k, stream);
      } else {
        usleep(0);
      }
    } while (count < rounds + concurrency);
    end = get_usec();

    for (int s = 1; s <= concurrency; s++)
      rsa->sync(s, true);

    double throughput = (k * 1000000.0) * (count - concurrency) / (end - begin);
    printf("%4d *%2d\t%.2f\n", k, concurrency, throughput);
  }
}

void test_rsa_cpu() {
  printf("------------------------------------------\n");
  printf("RSA1024, CPU, random\n");
  printf("------------------------------------------\n");
  rsa_context rsa1024_cpu(1024);
  test_latency(&rsa1024_cpu);
  test_correctness(&rsa1024_cpu, 20);

  printf("------------------------------------------\n");
  printf("RSA2048, CPU, random\n");
  printf("------------------------------------------\n");
  rsa_context rsa2048_cpu(2048);
  test_latency(&rsa2048_cpu);
  test_correctness(&rsa2048_cpu, 20);

  printf("------------------------------------------\n");
  printf("RSA4096, CPU, random\n");
  printf("------------------------------------------\n");
  rsa_context rsa4096_cpu(4096);
  test_latency(&rsa4096_cpu);
  test_correctness(&rsa4096_cpu, 20);
}

void test_rsa_rns(int bits) {
  // printf("------------------------------------------\n");
  // printf("RSA512, GPU (RNS), random\n");
  // printf("------------------------------------------\n");
  rsa_context_rns rsa_rns(bits);
  test_latency(&rsa_rns);
  // test_correctness(&rsa512_rns, 20);

  // printf("------------------------------------------\n");
  // printf("RSA1024, GPU (RNS), random\n");
  // printf("------------------------------------------\n");
  // rsa_context_rns rsa1024_rns(bits);
  // test_latency(&rsa1024_rns);
  // test_correctness(&rsa1024_rns, 20);

  // printf("------------------------------------------\n");
  // printf("RSA2048, GPU (RNS), random\n");
  // printf("------------------------------------------\n");
  // rsa_context_rns rsa2048_rns(2048);
  // test_latency(&rsa2048_rns);
  // test_correctness(&rsa2048_rns, 20);
}

void test_rsa_rns_fix(char *filename, char *password) {
  printf("fix mode.\n");
  rsa_context_rns rsa_rns(filename, password);
  test_latency(&rsa_rns);
}

void test_rsa_mp_fix(char* filename, char* password)
{
	printf("fix mode.\n");
	device_context dev_ctx;
	dev_ctx.init(10485760, 0);
	rsa_context_mp rsa_mp(filename, password);
	rsa_mp.set_device_context(&dev_ctx);
	test_latency(&rsa_mp);
}

void test_rsa_mp(int bits)
{
	device_context dev_ctx;
	dev_ctx.init(10485760, 0);

	rsa_context_mp rsa_mp(bits);
	rsa_mp.set_device_context(&dev_ctx);
	test_latency(&rsa_mp);

	// printf("------------------------------------------\n");
	// printf("RSA512, GPU (MP), random\n");
	// printf("------------------------------------------\n");
	// rsa_context_mp rsa512_mp(512);
	// rsa512_mp.set_device_context(&dev_ctx);
	// test_latency(&rsa512_mp);
	// test_correctness(&rsa512_mp, 20);

	// printf("------------------------------------------\n");
	// printf("RSA1024, GPU (MP), random\n");
	// printf("------------------------------------------\n");
	// rsa_context_mp rsa1024_mp(1024);
	// rsa1024_mp.set_device_context(&dev_ctx);
	// test_latency(&rsa1024_mp);
	// test_correctness(&rsa1024_mp, 20);

	// printf("------------------------------------------\n");
	// printf("RSA2048, GPU (MP), random\n");
	// printf("------------------------------------------\n");
	// rsa_context_mp rsa2048_mp(2048);
	// rsa2048_mp.set_device_context(&dev_ctx);
	// test_latency(&rsa2048_mp);
	// test_correctness(&rsa2048_mp, 20);

	// printf("------------------------------------------\n");
	// printf("RSA4096, GPU (MP), random\n");
	// printf("------------------------------------------\n");
	// rsa_context_mp rsa4096_mp(4096);
	// rsa4096_mp.set_device_context(&dev_ctx);
	// test_latency(&rsa4096_mp);
	// test_correctness(&rsa4096_mp, 20);
}
void test_rsa_mp_stream(unsigned num_stream) {
  device_context dev_ctx;
  dev_ctx.init(10485760, num_stream);

  printf("------------------------------------------\n");
  printf("RSA512, GPU (MP), random\n");
  printf("------------------------------------------\n");
  rsa_context_mp rsa512_mp(512);
  rsa512_mp.set_device_context(&dev_ctx);
  test_latency_stream(&rsa512_mp, &dev_ctx, num_stream);

  printf("------------------------------------------\n");
  printf("RSA1024, GPU (MP), random\n");
  printf("------------------------------------------\n");
  rsa_context_mp rsa1024_mp(1024);
  rsa1024_mp.set_device_context(&dev_ctx);
  test_latency_stream(&rsa1024_mp, &dev_ctx, num_stream);

  printf("------------------------------------------\n");
  printf("RSA2048, GPU (MP), random\n");
  printf("------------------------------------------\n");
  rsa_context_mp rsa2048_mp(2048);
  rsa2048_mp.set_device_context(&dev_ctx);
  test_latency_stream(&rsa2048_mp, &dev_ctx, num_stream);

  printf("------------------------------------------\n");
  printf("RSA4096, GPU (MP), random\n");
  printf("------------------------------------------\n");
  rsa_context_mp rsa4096_mp(4096);
  rsa4096_mp.set_device_context(&dev_ctx);
  test_latency_stream(&rsa4096_mp, &dev_ctx, num_stream);
}

void test_rsa_mp_cert(unsigned num_stream) {
  device_context dev_ctx;
  dev_ctx.init(10485760, num_stream);

  printf("------------------------------------------\n");
  printf("RSA1024, GPU (MP), server.key\n");
  printf("------------------------------------------\n");
  rsa_context_mp rsa("../../server.key", "anlab");
  rsa.set_device_context(&dev_ctx);
  // rsa.dump();
  if (num_stream == 0) {
    test_latency(&rsa);
    test_correctness(&rsa, 20);
  } else {
    for (unsigned int i = 1; i <= 16; i++)
      test_latency_stream(&rsa, &dev_ctx, i);
  }
}

static char usage[] =
    "Usage: %s -m MP,RNS,CPU [-s number of stream (MP-mode only)] "
    "[-f using fix key instead of random]"
    "[-b 512|1024|2048]\n";

bool fix = false;
int main(int argc, char *argv[]) {
  srand(time(NULL));
#if 0
	int count = 0;

	while (1) {
		mp_test_cpu();
		mp_test_gpu();

		count++;
		if (count % 1 == 0)
			printf("%d times...\n", count);
	}
#else
  bool mp = false;
  bool rns = false;
  bool cpu = false;

  int num_stream = 0;
  int bits = 512;
  int i = 1;
  while (i < argc) {
    if (strcmp(argv[i], "-m") == 0) {
      i++;
      if (i == argc)
        goto parse_error;

      if (strcmp(argv[i], "MP") == 0) {
        mp = true;
      } else if (strcmp(argv[i], "RNS") == 0) {
        rns = true;
      } else if (strcmp(argv[i], "CPU") == 0) {
        cpu = true;
      } else {
        goto parse_error;
      }
    } else if (strcmp(argv[i], "-s") == 0) {
      if (!mp)
        goto parse_error;
      i++;
      if (i == argc)
        goto parse_error;
      num_stream = atoi(argv[i]);
      if (num_stream > 16 || num_stream < 0)
        goto parse_error;
    } else if (strcmp(argv[i], "-b") == 0) {
      i++;
      if (i == argc)
        goto parse_error;
      bits = atoi(argv[i]);
      if (bits != 512 && bits != 1024 && bits != 2048)
        goto parse_error;
    } else if (strcmp(argv[i], "-f") == 0) {
      fix = true;
    } else {
      goto parse_error;
    }
    i++;
  }

  if (!(mp || rns || cpu))
    goto parse_error;

  char fix_private_key_file[40];
  sprintf(fix_private_key_file, "./bin/keys/private_key_%d.pem", bits);

  if (mp) {
    if (num_stream > 0) {
      test_rsa_mp_stream(num_stream);
    } else if (num_stream == 0) {
      device_context dev_ctx;
      dev_ctx.init(10485760, 0);

      rsa_context_mp rsa_mp =
          fix ? rsa_context_mp(fix_private_key_file, nullptr)
              : rsa_context_mp(bits);
      rsa_mp.set_device_context(&dev_ctx);
      test_latency(&rsa_mp);
    }
  }

  if (rns) {
    rsa_context_rns rsa_rns =
        fix ? rsa_context_rns(fix_private_key_file, nullptr)
            : rsa_context_rns(bits);

    test_latency(&rsa_rns);
  }

  if (cpu) {
    test_rsa_cpu();
  }

  return 0;

parse_error:
  printf(usage, argv[0]);
  return -1;

#endif
}
