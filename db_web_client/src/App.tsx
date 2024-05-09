import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { DBMainLayout } from './layouts/DBMainLayout'

const queryClient = new QueryClient()

function App() {

  return (
      <QueryClientProvider client={queryClient}>
        <DBMainLayout />
        <ReactQueryDevtools initialIsOpen={false} />
      </QueryClientProvider>
  )
}

export default App
