import type { AuthResponse } from '~/types'

export const useAuth = () => {
  const config = useRuntimeConfig()
  const token = useCookie('auth_token', {
    maxAge: 60 * 60 * 24 * 7, // 7 days
    sameSite: 'lax'
  })
  const user = useState<AuthResponse['user'] | null>('user', () => null)

  const isAuthenticated = computed(() => !!token.value)

  const signup = async (email: string, password: string) => {
    const { data, error } = await useFetch<AuthResponse>(`${config.public.apiBase}/auth/signup`, {
      method: 'POST',
      body: { email, password },
    })

    if (error.value) {
      throw new Error(error.value.data?.message || 'Signup failed')
    }

    if (data.value) {
      token.value = data.value.access_token
      user.value = data.value.user
    }

    return data.value
  }

  const login = async (email: string, password: string) => {
    const { data, error } = await useFetch<AuthResponse>(`${config.public.apiBase}/auth/login`, {
      method: 'POST',
      body: { email, password },
    })

    if (error.value) {
      throw new Error(error.value.data?.message || 'Login failed')
    }

    if (data.value) {
      token.value = data.value.access_token
      user.value = data.value.user
      console.log('Login successful, token set:', !!token.value)
    }

    return data.value
  }

  const logout = () => {
    token.value = null
    user.value = null
    navigateTo('/login')
  }

  const getAuthHeaders = () => {
    if (!token.value) return {}
    return {
      Authorization: `Bearer ${token.value}`,
    }
  }

  return {
    token: readonly(token),
    user: readonly(user),
    isAuthenticated,
    signup,
    login,
    logout,
    getAuthHeaders,
  }
}
