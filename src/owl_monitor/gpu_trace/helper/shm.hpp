

#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <vector>
#include <sys/ipc.h>
#include <sys/shm.h>
#include <unistd.h>
#include <sys/file.h>

using namespace std;

template <typename T>
class SharedMemory {
private:
    int shmid;
    key_t key;
    char *shm;
    size_t capacity;

public:
    SharedMemory(size_t capacity) : shmid(-1), key(-1), shm(nullptr), capacity(capacity) {
    }

    ~SharedMemory() {
        detach();
    }

    bool attach() {
        key = ftok(".", 'S');
        shmid = shmget(key, sizeof(uint64_t) + capacity * sizeof(T), 0666);
        if (shmid == -1) {
            perror("shmget");
            return false;
        }

        shm = (char*)shmat(shmid, nullptr, 0);
        if (shm == (char*)-1) {
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

    vector<T> parse() {
        
        
        
        
        

        uint64_t currentSize = *(uint64_t*)shm;

        T* data = (T*)(shm + sizeof(uint64_t));

        
        

        
        

        vector<T> v;
        
        for (uint64_t i=0; i<currentSize; i++) {
            
            v.push_back(data[i]);
        }

        return v;
        
        

        
        
        
        

        
    }
};