import type { Module, Job, CreateJobRequest, CreateJobResponse } from '~/types'

export const useApi = () => {
  const config = useRuntimeConfig()

  const getHeaders = () => {
    const token = useCookie('auth_token')
    if (!token.value) return {}
    return {
      Authorization: `Bearer ${token.value}`,
    }
  }

  const handleUnauthorized = () => {
    console.warn('Unauthorized request - logging out')
    const token = useCookie('auth_token')
    token.value = null
    navigateTo('/login')
  }

  const fetchModules = async (): Promise<Module[]> => {
    try {
      const data = await $fetch<Module[]>(`${config.public.apiBase}/modules`, {
        headers: getHeaders(),
      })
      return data
    } catch (error: any) {
      if (error.statusCode === 401 || error.statusCode === 403) {
        handleUnauthorized()
      }
      throw error
    }
  }

  const uploadModule = async (file: File, name: string): Promise<Module> => {
    const formData = new FormData()
    formData.append('file', file)
    formData.append('name', name)

    const response = await fetch(`${config.public.apiBase}/modules`, {
      method: 'POST',
      headers: getHeaders(),
      body: formData,
    })

    if (response.status === 401 || response.status === 403) {
      handleUnauthorized()
      throw new Error('Session expired. Please login again.')
    }

    if (!response.ok) {
      const error = await response.text()
      throw new Error(error || 'Failed to upload module')
    }

    return response.json()
  }

  const fetchJobs = async (): Promise<Job[]> => {
    try {
      const data = await $fetch<Job[]>(`${config.public.apiBase}/jobs`, {
        headers: getHeaders(),
      })
      return data
    } catch (error: any) {
      if (error.statusCode === 401 || error.statusCode === 403) {
        handleUnauthorized()
      }
      throw error
    }
  }

  const fetchJob = async (jobId: string): Promise<Job> => {
    try {
      const data = await $fetch<Job>(`${config.public.apiBase}/jobs/${jobId}`, {
        headers: getHeaders(),
      })
      return data
    } catch (error: any) {
      if (error.statusCode === 401 || error.statusCode === 403) {
        handleUnauthorized()
      }
      throw error
    }
  }

  const createJob = async (request: CreateJobRequest): Promise<CreateJobResponse> => {
    try {
      const data = await $fetch<CreateJobResponse>(`${config.public.apiBase}/jobs`, {
        method: 'POST',
        headers: {
          ...getHeaders(),
          'Content-Type': 'application/json',
        },
        body: request,
      })
      return data
    } catch (error: any) {
      if (error.statusCode === 401 || error.statusCode === 403) {
        handleUnauthorized()
      }
      throw error
    }
  }

  return {
    fetchModules,
    uploadModule,
    fetchJobs,
    fetchJob,
    createJob,
  }
}
