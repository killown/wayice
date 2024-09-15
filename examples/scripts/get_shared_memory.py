import mmap
import posix_ipc
import traceback

SHM_NAME_METHOD = "/wayice_list_windows"

def read_from_shared_memory(shm_name):
    try:
        shm = posix_ipc.SharedMemory(shm_name, posix_ipc.O_RDONLY)
        size = shm.size
        with mmap.mmap(shm.fd, size, access=mmap.ACCESS_READ) as shared_mem:
            shared_string = shared_mem.readline().decode('utf-8').rstrip('\x00')
            print(shared_string)
    except posix_ipc.ExistentialError:
        print("Failed to open shared memory!")
        return 1
    except Exception as e:
        print("An error occurred:")
        traceback.print_exc()
        return 1

    return 0

result = read_from_shared_memory(SHM_NAME_METHOD)
if result != 0:
    print(f"Process exited with error code: {result}")
