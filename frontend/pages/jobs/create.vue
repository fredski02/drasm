<template>
  <div>
    <div class="max-w-2xl mx-auto">
      <div class="mb-6">
        <NuxtLink to="/jobs" class="text-primary-600 hover:text-primary-700">
          ← Back to Jobs
        </NuxtLink>
      </div>
      
      <div class="bg-white rounded-lg shadow-md p-8">
        <h2 class="text-2xl font-bold text-gray-900 mb-6">Create Job</h2>
        
        <form @submit.prevent="handleSubmit" class="space-y-6">
          <div>
            <label for="module" class="block text-sm font-medium text-gray-700 mb-1">
              Select Module
            </label>
            <select
              id="module"
              v-model="selectedModuleId"
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="">Choose a module...</option>
              <option v-for="module in modules" :key="module.id" :value="module.id">
                {{ module.name }} ({{ module.hash.substring(0, 8) }}...)
              </option>
            </select>
            <p class="mt-1 text-sm text-gray-500">
              Select the WASM module to execute
            </p>
          </div>
          
          <div>
            <label for="typeName" class="block text-sm font-medium text-gray-700 mb-1">
              Type Name
            </label>
            <input
              id="typeName"
              v-model="typeName"
              type="text"
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
              placeholder="e.g., Request, EchoRequest"
            />
            <p class="mt-1 text-sm text-gray-500">
              The request type expected by the WASM module
            </p>
          </div>
          
          <div>
            <label for="payload" class="block text-sm font-medium text-gray-700 mb-1">
              Payload (JSON)
            </label>
            <textarea
              id="payload"
              v-model="payload"
              rows="6"
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 font-mono text-sm"
              placeholder='{"data": "Hello from DRASM!"}'
            />
            <p class="mt-1 text-sm text-gray-500">
              JSON payload that matches your module's request structure
            </p>
          </div>
          
          <div v-if="error" class="bg-red-50 border border-red-200 rounded-md p-4">
            <p class="text-sm text-red-800">{{ error }}</p>
          </div>
          
          <div v-if="success" class="bg-green-50 border border-green-200 rounded-md p-4">
            <p class="text-sm text-green-800">
              Job created successfully!
              <NuxtLink :to="`/jobs/${createdJobId}`" class="font-medium underline">
                View job
              </NuxtLink>
            </p>
          </div>
          
          <div class="flex space-x-4">
            <button
              type="submit"
              :disabled="loading || !selectedModuleId"
              class="flex-1 px-4 py-2 text-white bg-primary-600 rounded-md hover:bg-primary-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
            >
              {{ loading ? 'Creating...' : 'Create Job' }}
            </button>
            <NuxtLink
              to="/jobs"
              class="flex-1 px-4 py-2 text-center text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200"
            >
              Cancel
            </NuxtLink>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Module } from '~/types'

definePageMeta({
  middleware: 'auth'
})

const { fetchModules, createJob } = useApi()
const route = useRoute()
const router = useRouter()

const modules = ref<Module[]>([])
const selectedModuleId = ref('')
const typeName = ref('Request')
const payload = ref('{"data": "Hello from DRASM!"}')
const error = ref('')
const success = ref(false)
const loading = ref(false)
const createdJobId = ref('')

// Load modules on mount
onMounted(async () => {
  try {
    modules.value = await fetchModules()
    
    // Pre-select module if passed in query params
    if (route.query.module) {
      selectedModuleId.value = route.query.module as string
    }
  } catch (e: any) {
    error.value = 'Failed to load modules: ' + e.message
  }
})

const handleSubmit = async () => {
  error.value = ''
  success.value = false
  
  // Validate JSON
  try {
    JSON.parse(payload.value)
  } catch (e) {
    error.value = 'Invalid JSON payload'
    return
  }
  
  loading.value = true
  
  try {
    const response = await createJob({
      module_id: selectedModuleId.value,
      message: {
        type_name: typeName.value,
        payload: payload.value
      }
    })
    
    success.value = true
    createdJobId.value = response.job_id
    
    setTimeout(() => {
      router.push(`/jobs/${response.job_id}`)
    }, 1500)
  } catch (e: any) {
    error.value = e.message || 'Failed to create job'
  } finally {
    loading.value = false
  }
}
</script>
