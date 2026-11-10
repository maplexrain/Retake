import { useState, useRef, useEffect, ReactNode } from 'react'
import { ChevronDownIcon } from './icons'

export interface SelectOption<T = string | number> {
  value: T
  label: string
  sublabel?: string
  badge?: string
  disabled?: boolean
  icon?: ReactNode
}

interface CustomSelectProps<T = string | number> {
  value: T
  options: SelectOption<T>[]
  onChange: (value: T) => void
  placeholder?: string
  disabled?: boolean
  className?: string
  menuClassName?: string
  icon?: ReactNode
}

export function CustomSelect<T = string | number>({
  value,
  options,
  onChange,
  placeholder,
  disabled,
  className = '',
  menuClassName = '',
  icon,
}: CustomSelectProps<T>) {
  const [isOpen, setIsOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  const selectedOption = options.find(o => o.value === value)

  useEffect(() => {
    if (!isOpen) return
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false)
      }
    }
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setIsOpen(false)
    }
    document.addEventListener('mousedown', handleClickOutside)
    window.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [isOpen])

  return (
    <div
      ref={containerRef}
      className={`custom-select-container ${disabled ? 'disabled' : ''} ${isOpen ? 'open' : ''} ${className}`}
    >
      <button
        type="button"
        className="custom-select-trigger"
        onClick={() => !disabled && setIsOpen(v => !v)}
        disabled={disabled}
      >
        <div className="custom-select-trigger-left">
          {icon && <span className="custom-select-icon">{icon}</span>}
          {selectedOption?.icon && <span className="custom-select-icon">{selectedOption.icon}</span>}
          <span className="custom-select-value">
            {selectedOption ? selectedOption.label : placeholder || 'Select...'}
          </span>
          {selectedOption?.badge && (
            <span className="custom-select-badge">{selectedOption.badge}</span>
          )}
        </div>
        <ChevronDownIcon size={12} className={`custom-select-arrow ${isOpen ? 'rotated' : ''}`} />
      </button>

      {isOpen && (
        <div className={`custom-select-menu ${menuClassName}`}>
          {options.map((option, idx) => {
            const isSelected = option.value === value
            return (
              <div
                key={String(option.value) + '_' + idx}
                className={`custom-select-item ${isSelected ? 'selected' : ''} ${option.disabled ? 'disabled' : ''}`}
                onClick={() => {
                  if (option.disabled) return
                  onChange(option.value)
                  setIsOpen(false)
                }}
              >
                <div className="custom-select-item-left">
                  {option.icon && <span className="custom-select-item-icon">{option.icon}</span>}
                  <span className="custom-select-item-label">{option.label}</span>
                  {option.badge && <span className="custom-select-item-badge">{option.badge}</span>}
                </div>
                {option.sublabel && (
                  <span className="custom-select-item-sublabel">{option.sublabel}</span>
                )}
                {isSelected && <span className="custom-select-check">✓</span>}
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
