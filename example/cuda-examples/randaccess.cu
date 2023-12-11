#include <cstdlib>
#include <ctime>
#include <iostream>

__global__ void randomAccessKernel(int **array, int rows, int cols,
                                   int randValue, int randValue2) {
  int tid = blockIdx.x * blockDim.x + threadIdx.x;
  int row = (threadIdx.x * 2 + tid + randValue) % rows;
  int col = ((threadIdx.x + 1) * 3 + tid + randValue2) % cols;

  if (row < rows && col < cols) {
    array[row][col] = tid;
  }
}

int main(int argc, char *argv[]) {
  int rows = 100;
  int cols = 100;
  int numThreads = 1024;

  if (argc > 1) {
    numThreads = std::atoi(argv[1]);
  }

  std::srand(std::time(0));

  int **array;
  cudaMallocManaged(&array, rows * sizeof(int *));
  for (int i = 0; i < rows; i++) {
    cudaMallocManaged(&array[i], cols * sizeof(int));
  }

  int numBlocks = (numThreads + 255) / 256;
  dim3 gridDim(numBlocks, 1, 1);
  dim3 blockDim(256, 1, 1);

  int randValue = std::rand();
  int randValue2 = std::rand();
  std::cout << randValue << " " << randValue2 << std::endl;

  randomAccessKernel<<<gridDim, blockDim>>>(array, rows, cols, randValue,
                                            randValue2);
  cudaDeviceSynchronize();

  for (int i = 0; i < rows; i++) {
    cudaFree(array[i]);
  }
  cudaFree(array);

  return 0;
}
