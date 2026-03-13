import { createHashRouter } from 'react-router-dom'
import { AppShell } from '../components/layout/AppShell'
import { MarketPage } from '../pages/MarketPage'
import { RepositoryPage } from '../pages/RepositoryPage'
import { SecurityPage } from '../pages/SecurityPage'
import { SettingsPage } from '../pages/SettingsPage'
import { SkillsPage } from '../pages/SkillsPage'
import { TemplatesPage } from '../pages/TemplatesPage'

export const router = createHashRouter([
  {
    path: '/',
    element: <AppShell />,
    children: [
      { index: true, element: <RepositoryPage /> },
      { path: 'skills', element: <SkillsPage /> },
      { path: 'market', element: <MarketPage /> },
      { path: 'security', element: <SecurityPage /> },
      { path: 'settings', element: <SettingsPage /> },
      { path: 'templates', element: <TemplatesPage /> },
    ],
  },
])
