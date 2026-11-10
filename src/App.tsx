import Main from './ui/Main'
import Selector from './ui/Selector'
import Countdown from './ui/Countdown'
import FloatingHud from './ui/FloatingHud'
import ScopePreview from './ui/ScopePreview'

export default function App() {
  const hash = window.location.hash
  if (hash.includes('countdown')) return <Countdown />
  if (hash.includes('selector')) return <Selector />
  if (hash.includes('hud')) return <FloatingHud />
  if (hash.includes('scope-preview')) return <ScopePreview />
  return <Main />
}
