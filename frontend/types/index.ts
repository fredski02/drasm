export interface AuthResponse {
  access_token: string
  user: any
}

export interface User {
  id: string
  email: string
  created_at: string
}

export interface Module {
  id: string
  user_id: string
  name: string
  storage_path: string
  hash: string
  size_bytes: number
  created_at?: string
  updated_at?: string
}

export interface Job {
  job_id: string
  user_id: string
  module_id: string
  status: 'pending' | 'processing' | 'completed' | 'failed'
  input_message: MessageEnvelope
  result?: any
  error?: string
  worker_id?: string
  created_at: string
  completed_at?: string
}

export interface MessageEnvelope {
  type_name: string
  payload: number[]
}

export interface CreateJobRequest {
  module_id: string
  message: {
    type_name: string
    payload: string
  }
}

export interface CreateJobResponse {
  job_id: string
  status: string
}
