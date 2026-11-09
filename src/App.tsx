import Main from './ui/Main'
import Selector from './ui/Selector'
import Countdown from './ui/Countdown'
import FloatingHud from './ui/FloatingHud'

export default function App() {
  const hash = window.location.hash
  if (hash.includes('countdown')) return <Countdown />
  if (hash.includes('selector')) return <Selector />
  if (hash.includes('hud')) return <FloatingHud />
  return <Main />
}
