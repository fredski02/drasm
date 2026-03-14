<template>
  <div class="px-4 sm:px-6 lg:px-8">
    <div class="max-w-2xl mx-auto">
      <div class="mb-8">
        <h1 class="text-3xl font-bold text-gray-900">Upload Module</h1>
        <p class="mt-2 text-sm text-gray-700">
          Upload a new WebAssembly module to the platform
        </p>
      </div>

      <form @submit.prevent="handleUpload" class="space-y-6">
        <div v-if="error" class="rounded-md bg-red-50 p-4">
          <div class="flex">
            <div class="ml-3">
              <h3 class="text-sm font-medium text-red-800">{{ error }}</h3>
            </div>
          </div>
        </div>

        <div v-if="success" class="rounded-md bg-green-50 p-4">
          <div class="flex">
            <div class="ml-3">
              <h3 class="text-sm font-medium text-green-800">Module uploaded successfully!</h3>
            </div>
          </div>
        </div>

        <div>
          <label for="name" class="block text-sm font-medium text-gray-700">
            Module Name
          </label>
          <input
            id="name"
            v-model="form.name"
            type="text"
            required
            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
          />
        </div>

        <div>
          <label for="version" class="block text-sm font-medium text-gray-700">
            Version
          </label>
          <input
            id="version"
            v-model="form.version"
            type="text"
            required
            placeholder="1.0.0"
            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
          />
        </div>

        <div>
          <label for="description" class="block text-sm font-medium text-gray-700">
            Description
          </label>
          <textarea
            id="description"
            v-model="form.description"
            rows="3"
            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700">
            WASM File
          </label>
          <div class="mt-1 flex justify-center px-6 pt-5 pb-6 border-2 border-gray-300 border-dashed rounded-md">
            <div class="space-y-1 text-center">
              <svg
                class="mx-auto h-12 w-12 text-gray-400"
                stroke="currentColor"
                fill="none"
                viewBox="0 0 48 48"
                aria-hidden="true"
              >
                <path
                  d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
              <div class="flex text-sm text-gray-600">
                <label
                  for="file-upload"
                  class="relative cursor-pointer bg-white rounded-md font-medium text-indigo-600 hover:text-indigo-500 focus-within:outline-none focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-indigo-500"
                >
                  <span>Upload a file</span>
                  <input
                    id="file-upload"
                    name="file-upload"
                    type="file"
                    accept=".wasm"
                    class="sr-only"
                    @change="handleFileChange"
                    required
                  />
                </label>
                <p class="pl-1">or drag and drop</p>
              </div>
              <p class="text-xs text-gray-500">WASM files only</p>
              <p v-if="file" class="text-sm text-gray-900 mt-2">
                Selected: {{ file.name }}
              </p>
            </div>
          </div>
        </div>

        <div class="flex justify-end space-x-3">
          <NuxtLink
            to="/modules"
            class="inline-flex justify-center rounded-md border border-gray-300 bg-white py-2 px-4 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
          >
            Cancel
          </NuxtLink>
          <button
            type="submit"
            :disabled="loading"
            class="inline-flex justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50"
          >
            {{ loading ? 'Uploading...' : 'Upload' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  middleware: 'auth',
});

const { apiFetch } = useApi();
const router = useRouter();

const form = reactive({
  name: '',
  version: '',
  description: '',
});

const file = ref<File | null>(null);
const loading = ref(false);
const error = ref('');
const success = ref(false);

const handleFileChange = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    file.value = target.files[0];
  }
};

const handleUpload = async () => {
  if (!file.value) {
    error.value = 'Please select a file';
    return;
  }

  loading.value = true;
  error.value = '';
  success.value = false;

  try {
    const formData = new FormData();
    formData.append('name', form.name);
    formData.append('version', form.version);
    if (form.description) {
      formData.append('description', form.description);
    }
    formData.append('file', file.value);

    await apiFetch('/modules', {
      method: 'POST',
      body: formData,
      headers: {}, // Let browser set Content-Type with boundary
    });

    success.value = true;
    setTimeout(() => {
      router.push('/modules');
    }, 1500);
  } catch (e: any) {
    error.value = e.data?.message || 'Failed to upload module';
  } finally {
    loading.value = false;
  }
};
</script>
