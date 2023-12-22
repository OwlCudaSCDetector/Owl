#ifndef SHM_HPP
#define SHM_HPP

#include <iostream>
#include <memory>
#include <sys/ipc.h>
#include <sys/shm.h>
#include <vector>
/*
****************************shared memory************************************
*/

using namespace std;

template <typename T> class SharedMemory {
private:
  int shmid;
  key_t key;
  char *shm;
  size_t capacity;

public:
  SharedMemory(size_t capacity)
      : shmid(-1), key(-1), shm(nullptr), capacity(capacity) {}

  ~SharedMemory() { detach(); }

  bool create() {
    key = ftok(".", 'S');
    shmid =
        shmget(key, sizeof(uint64_t) + capacity * sizeof(T), IPC_CREAT | 0666);
    if (shmid == -1) {
      perror("shmget");
      return false;
    }

    shm = (char *)shmat(shmid, nullptr, 0);
    if (shm == (char *)-1) {
      perror("shmat");
      return false;
    }

    *((uint64_t *)shm) = 0;

    return true;
  }

  bool attach() {
    key = ftok(".", 'S');
    shmid = shmget(key, sizeof(uint64_t) + capacity * sizeof(T), 0666);
    if (shmid == -1) {
      perror("shmget");
      return false;
    }

    shm = (char *)shmat(shmid, nullptr, 0);
    if (shm == (char *)-1) {
      perror("shmat");
      return false;
    }

    return true;
  }

  bool detach() {
    if (shm != nullptr) {
      if (shmdt(shm) == -1) {
        perror("shmdt");
        return false;
      }
      shm = nullptr;
    }

    return true;
  }

  bool remove() {
    if (shmid != -1) {
      if (shmctl(shmid, IPC_RMID, nullptr) == -1) {
        perror("shmctl");
        return false;
      }
      shmid = -1;
    }

    return true;
  }

  void push(const T &data) {
    uint64_t *sizePtr = (uint64_t *)shm;

    T *dataArray = (T *)(shm + sizeof(uint64_t));

    if (*sizePtr >= capacity) {
      std::cerr << "Stack is full. Cannot push more data." << std::endl;
      return;
    }

    dataArray[*sizePtr] = data;
    (*sizePtr)++;
  }

  void write(vector<T *> datas) {
    uint64_t *sizePtr = (uint64_t *)shm;
    *sizePtr = datas.size();

    T *dataArray = (T *)(shm + sizeof(uint64_t));
    int i = 0;
    for (auto data : datas) {
      memcpy(dataArray + i++, (void *)data, sizeof(T));
    }
  }

  T pop() {
    uint64_t *sizePtr = (uint64_t *)shm;
    T *dataArray = (T *)(shm + sizeof(uint64_t));

    if (*sizePtr == 0) {
      std::cerr << "Stack is empty. Cannot pop data." << std::endl;
      return T();
    }

    T data = dataArray[*sizePtr - 1];
    (*sizePtr)--;

    return data;
  }
};

#endif