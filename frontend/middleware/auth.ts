export default defineNuxtRouteMiddleware((to, from) => {
  const token = useCookie('auth_token')
  
  console.log('Auth middleware - token exists:', !!token.value, 'path:', to.path)
  
  if (!token.value) {
    return navigateTo('/login')
  }
})
