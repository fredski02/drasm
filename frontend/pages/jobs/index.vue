<template>
  <div>
    <div class="mb-6 flex justify-between items-center">
      <h1 class="text-3xl font-bold text-gray-900">Jobs</h1>
      <NuxtLink
        to="/jobs/create"
        class="px-4 py-2 text-white bg-primary-600 rounded-md hover:bg-primary-700"
      >
        Create Job
      </NuxtLink>
    </div>
    
    <div v-if="loading" class="text-center py-12">
      <p class="text-gray-600">Loading jobs...</p>
    </div>
    
    <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-md p-4">
      <p class="text-red-800">{{ error }}</p>
    </div>
    
    <div v-else-if="jobs.length === 0" class="text-center py-12">
      <p class="text-gray-600 mb-4">No jobs created yet</p>
      <NuxtLink
        to="/jobs/create"
        class="inline-block px-4 py-2 text-white bg-primary-600 rounded-md hover:bg-primary-700"
      >
        Create Your First Job
      </NuxtLink>
    </div>
    
    <div v-else class="bg-white shadow-md rounded-lg overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Job ID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Module
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Status
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Created
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          <tr v-for="job in jobs" :key="job.job_id">
            <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">
              {{ job.job_id.substring(0, 8) }}...
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ job.module_id.substring(0, 8) }}...
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <span :class="getStatusClass(job.status)">
                {{ job.status }}
              </span>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ formatDate(job.created_at) }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm">
              <NuxtLink
                :to="`/jobs/${job.job_id}`"
                class="text-primary-600 hover:text-primary-700 font-medium"
              >
                View Details
              </NuxtLink>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    
    <div v-if="jobs.length > 0" class="mt-4 text-center">
      <button
        @click="refresh"
        class="px-4 py-2 text-sm text-primary-600 hover:text-primary-700 font-medium"
      >
        Refresh
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Job } from '~/types'

definePageMeta({
  middleware: 'auth'
})

const { fetchJobs } = useApi()

const jobs = ref<Job[]>([])
const loading = ref(true)
const error = ref('')

const getStatusClass = (status: string): string => {
  const classes: Record<string, string> = {
    pending: 'px-2 py-1 text-xs font-medium bg-yellow-100 text-yellow-800 rounded',
    processing: 'px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 rounded',
    completed: 'px-2 py-1 text-xs font-medium bg-green-100 text-green-800 rounded',
    failed: 'px-2 py-1 text-xs font-medium bg-red-100 text-red-800 rounded',
  }
  return classes[status] || 'px-2 py-1 text-xs font-medium bg-gray-100 text-gray-800 rounded'
}

const formatDate = (dateString: string): string => {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

const loadJobs = async () => {
  loading.value = true
  error.value = ''
  try {
    jobs.value = await fetchJobs()
  } catch (e: any) {
    error.value = e.message || 'Failed to load jobs'
  } finally {
    loading.value = false
  }
}

const refresh = () => {
  loadJobs()
}

onMounted(() => {
  loadJobs()
})
</script>
