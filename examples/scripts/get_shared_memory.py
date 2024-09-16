import mmap
import posix_ipc
import traceback

SEM_NAME = "/semaphore_wayice_list_windows"
SHM_NAME = "/wayice_list_windows"

def read_from_shared_memory(shm_name, sem_name):
    try:
        # Open the semaphore
        semaphore = posix_ipc.Semaphore(sem_name, posix_ipc.O_CREAT)

        # Wait (acquire semaphore)
        semaphore.acquire()

        # Open the shared memory
        shm = posix_ipc.SharedMemory(shm_name, posix_ipc.O_RDONLY)
        size = shm.size

        # Map the shared memory
        with mmap.mmap(shm.fd, size, access=mmap.ACCESS_READ) as shared_mem:
            shared_string = shared_mem.readline().decode('utf-8').rstrip('\x00')
            print("Shared Memory Content:", shared_string)

        # Release (post) the semaphore
        semaphore.release()

    except posix_ipc.ExistentialError:
        print("Failed to open shared memory or semaphore!")
        return 1
    except Exception as e:
        print(f"An error occurred: {e}")
        traceback.print_exc()
        return 1

    return 0

result = read_from_shared_memory(SHM_NAME, SEM_NAME)
if result != 0:
    print(f"Process exited with error code: {result}")
