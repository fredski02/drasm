<template>
  <div>
    <div class="mb-6">
      <NuxtLink to="/jobs" class="text-primary-600 hover:text-primary-700">
        ← Back to Jobs
      </NuxtLink>
    </div>
    
    <div v-if="loading" class="text-center py-12">
      <p class="text-gray-600">Loading job details...</p>
    </div>
    
    <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-md p-4">
      <p class="text-red-800">{{ error }}</p>
    </div>
    
    <div v-else-if="job" class="space-y-6">
      <div class="bg-white rounded-lg shadow-md p-6">
        <div class="flex justify-between items-start mb-4">
          <h1 class="text-2xl font-bold text-gray-900">Job Details</h1>
          <span :class="getStatusClass(job.status)">
            {{ job.status }}
          </span>
        </div>
        
        <dl class="grid grid-cols-1 gap-4">
          <div>
            <dt class="text-sm font-medium text-gray-500">Job ID</dt>
            <dd class="mt-1 text-sm text-gray-900 font-mono">{{ job.job_id }}</dd>
          </div>
          
          <div>
            <dt class="text-sm font-medium text-gray-500">Module ID</dt>
            <dd class="mt-1 text-sm text-gray-900 font-mono">{{ job.module_id }}</dd>
          </div>
          
          <div>
            <dt class="text-sm font-medium text-gray-500">Created At</dt>
            <dd class="mt-1 text-sm text-gray-900">{{ formatDate(job.created_at) }}</dd>
          </div>
          
          <div v-if="job.completed_at">
            <dt class="text-sm font-medium text-gray-500">Completed At</dt>
            <dd class="mt-1 text-sm text-gray-900">{{ formatDate(job.completed_at) }}</dd>
          </div>
          
          <div v-if="job.worker_id">
            <dt class="text-sm font-medium text-gray-500">Worker ID</dt>
            <dd class="mt-1 text-sm text-gray-900 font-mono">{{ job.worker_id }}</dd>
          </div>
        </dl>
      </div>
      
      <div class="bg-white rounded-lg shadow-md p-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">Input Message</h2>
        <div class="bg-gray-50 rounded-md p-4">
          <pre class="text-sm text-gray-900 overflow-x-auto">{{ formatJson(job.input_message) }}</pre>
        </div>
      </div>
      
      <div v-if="job.result" class="bg-white rounded-lg shadow-md p-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">Result</h2>
        <div class="bg-gray-50 rounded-md p-4">
          <pre class="text-sm text-gray-900 overflow-x-auto">{{ formatJson(job.result) }}</pre>
        </div>
      </div>
      
      <div v-if="job.error" class="bg-red-50 border border-red-200 rounded-md p-6">
        <h2 class="text-lg font-semibold text-red-900 mb-4">Error</h2>
        <p class="text-sm text-red-800">{{ job.error }}</p>
      </div>
      
      <div class="flex justify-center">
        <button
          @click="refresh"
          :disabled="refreshing"
          class="px-4 py-2 text-sm text-primary-600 hover:text-primary-700 font-medium disabled:text-gray-400"
        >
          {{ refreshing ? 'Refreshing...' : 'Refresh Status' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Job } from '~/types'

definePageMeta({
  middleware: 'auth'
})

const route = useRoute()
const { fetchJob } = useApi()

const job = ref<Job | null>(null)
const loading = ref(true)
const error = ref('')
const refreshing = ref(false)

const jobId = computed(() => route.params.id as string)

const getStatusClass = (status: string): string => {
  const classes: Record<string, string> = {
    pending: 'px-3 py-1 text-sm font-medium bg-yellow-100 text-yellow-800 rounded',
    processing: 'px-3 py-1 text-sm font-medium bg-blue-100 text-blue-800 rounded',
    completed: 'px-3 py-1 text-sm font-medium bg-green-100 text-green-800 rounded',
    failed: 'px-3 py-1 text-sm font-medium bg-red-100 text-red-800 rounded',
  }
  return classes[status] || 'px-3 py-1 text-sm font-medium bg-gray-100 text-gray-800 rounded'
}

const formatDate = (dateString: string): string => {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

const formatJson = (obj: any): string => {
  try {
    return JSON.stringify(obj, null, 2)
  } catch (e) {
    return String(obj)
  }
}

const loadJob = async () => {
  loading.value = true
  error.value = ''
  try {
    job.value = await fetchJob(jobId.value)
  } catch (e: any) {
    error.value = e.message || 'Failed to load job'
  } finally {
    loading.value = false
  }
}

const refresh = async () => {
  refreshing.value = true
  try {
    job.value = await fetchJob(jobId.value)
  } catch (e: any) {
    error.value = e.message || 'Failed to refresh job'
  } finally {
    refreshing.value = false
  }
}

onMounted(() => {
  loadJob()
})
</script>
