import { Node, mergeAttributes } from '@tiptap/core'

export interface AudioOptions {
  HTMLAttributes: Record<string, any>
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    audio: {
      /**
       * Insert audio with URL
       */
      setAudio: (options: { src: string; title?: string }) => ReturnType
      /**
       * Update audio attributes
       */
      updateAudio: (options: { src?: string; title?: string }) => ReturnType
    }
  }
}

export const Audio = Node.create<AudioOptions>({
  name: 'audio',

  group: 'inline',

  inline: true,

  atom: true,

  selectable: true,

  addOptions() {
    return {
      HTMLAttributes: {},
    }
  },

  addAttributes() {
    return {
      src: {
        default: null,
        parseHTML: (element) => element.getAttribute('data-src'),
        renderHTML: (attributes) => {
          if (!attributes.src) {
            return {}
          }
          return {
            'data-src': attributes.src,
          }
        },
      },
      title: {
        default: null,
        parseHTML: (element) => element.getAttribute('data-title'),
        renderHTML: (attributes) => {
          if (!attributes.title) {
            return {}
          }
          return {
            'data-title': attributes.title,
          }
        },
      },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'span[data-type="audio"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'span',
      mergeAttributes(HTMLAttributes, {
        'data-type': 'audio',
        class: 'inline-block mx-1'
      }),
      [
        'button',
        {
          class: 'inline-flex items-center gap-2 px-3 py-1.5 bg-primary text-primary-foreground border-0 rounded-md cursor-pointer transition-all duration-200 hover:bg-primary/90 active:scale-95',
          type: 'button',
          'data-src': HTMLAttributes['data-src'],
          'title': HTMLAttributes['data-title'] || 'Play audio',
        },
        [
          'svg',
          {
            xmlns: 'http://www.w3.org/2000/svg',
            width: '16',
            height: '16',
            viewBox: '0 0 24 24',
            fill: 'none',
            stroke: 'currentColor',
            'stroke-width': '2',
            'stroke-linecap': 'round',
            'stroke-linejoin': 'round',
          },
          [
            'polygon',
            {
              points: '11 5 6 9 2 9 2 15 6 15 11 19 11 5',
            },
          ],
          [
            'path',
            {
              d: 'M15.54 8.46a5 5 0 0 1 0 7.07',
            },
          ],
        ],
        [
          'span',
          {
            class: 'audio-label text-sm font-medium'
          },
          'Phát audio'
        ],
      ],
    ]
  },

  addCommands() {
    return {
      setAudio:
        (options) =>
        ({ commands }) => {
          return commands.insertContent({
            type: this.name,
            attrs: options,
          })
        },
      updateAudio:
        (options) =>
        ({ commands }) => {
          return commands.updateAttributes(this.name, options)
        },
    }
  },

  addNodeView() {
    return ({ node, editor }) => {
      const dom = document.createElement('span')
      dom.setAttribute('data-type', 'audio')
      dom.className = 'inline-block mx-1'

      const button = document.createElement('button')
      button.type = 'button'
      button.className = 'inline-flex items-center gap-2 px-3 py-1.5 bg-primary text-primary-foreground border-0 rounded-md cursor-pointer transition-all duration-200 hover:bg-primary/90 active:scale-95'
      button.title = node.attrs.title || 'Play audio'

      // Create Volume icon (stopped state - no waves)
      const createVolumeIcon = () => {
        const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg')
        svg.setAttribute('width', '16')
        svg.setAttribute('height', '16')
        svg.setAttribute('viewBox', '0 0 24 24')
        svg.setAttribute('fill', 'none')
        svg.setAttribute('stroke', 'currentColor')
        svg.setAttribute('stroke-width', '2')
        svg.setAttribute('stroke-linecap', 'round')
        svg.setAttribute('stroke-linejoin', 'round')

        // Speaker polygon
        const polygon = document.createElementNS('http://www.w3.org/2000/svg', 'polygon')
        polygon.setAttribute('points', '11 5 6 9 2 9 2 15 6 15 11 19 11 5')

        svg.appendChild(polygon)
        return svg
      }

      // Create Volume2 icon (playing state - with waves)
      const createVolume2Icon = () => {
        const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg')
        svg.setAttribute('width', '16')
        svg.setAttribute('height', '16')
        svg.setAttribute('viewBox', '0 0 24 24')
        svg.setAttribute('fill', 'none')
        svg.setAttribute('stroke', 'currentColor')
        svg.setAttribute('stroke-width', '2')
        svg.setAttribute('stroke-linecap', 'round')
        svg.setAttribute('stroke-linejoin', 'round')

        // Speaker polygon
        const polygon = document.createElementNS('http://www.w3.org/2000/svg', 'polygon')
        polygon.setAttribute('points', '11 5 6 9 2 9 2 15 6 15 11 19 11 5')

        // Sound wave 1
        const path1 = document.createElementNS('http://www.w3.org/2000/svg', 'path')
        path1.setAttribute('d', 'M15.54 8.46a5 5 0 0 1 0 7.07')

        // Sound wave 2
        const path2 = document.createElementNS('http://www.w3.org/2000/svg', 'path')
        path2.setAttribute('d', 'M19.07 4.93a10 10 0 0 1 0 14.14')

        svg.appendChild(polygon)
        svg.appendChild(path1)
        svg.appendChild(path2)
        return svg
      }

      // Create text label
      const textLabel = document.createElement('span')
      textLabel.className = 'audio-label text-sm font-medium'
      textLabel.textContent = 'Phát audio'

      // Start with volume icon (stopped) and text
      button.appendChild(createVolumeIcon())
      button.appendChild(textLabel)

      dom.appendChild(button)

      // Audio playback logic
      let audioPlayer: HTMLAudioElement | null = null
      let isPlaying = false

      const updateIcon = (playing: boolean) => {
        // Clear current content
        button.innerHTML = ''

        // Add appropriate icon
        if (playing) {
          button.appendChild(createVolume2Icon())
          textLabel.textContent = 'Tắt audio'
          button.title = 'Stop audio'
        } else {
          button.appendChild(createVolumeIcon())
          textLabel.textContent = 'Phát audio'
          button.title = 'Play audio'
        }

        // Re-add text label
        button.appendChild(textLabel)
      }

      const playAudio = async () => {
        const src = node.attrs.src
        if (!src) {
          console.error('No audio source provided')
          return
        }

        // If already playing, stop it
        if (isPlaying && audioPlayer) {
          audioPlayer.pause()
          audioPlayer.currentTime = 0
          isPlaying = false
          button.classList.remove('bg-destructive')
          button.classList.add('bg-primary')
          updateIcon(false)
          return
        }

        // Create new audio element if needed
        if (!audioPlayer) {
          audioPlayer = new window.Audio()
          audioPlayer.preload = 'auto'

          audioPlayer.addEventListener('ended', () => {
            isPlaying = false
            button.classList.remove('bg-destructive')
            button.classList.add('bg-primary')
            updateIcon(false)
          })

          audioPlayer.addEventListener('error', (e) => {
            console.error('Error loading audio:', src, e)
            const errorMessage = audioPlayer?.error
            if (errorMessage) {
              console.error('Error code:', errorMessage.code, 'Message:', errorMessage.message)
            }
            isPlaying = false
            button.classList.remove('bg-destructive')
            button.classList.add('bg-primary')
            updateIcon(false)

            // Show error to user
            if (typeof window !== 'undefined') {
              alert('Không thể phát audio. Vui lòng kiểm tra URL hoặc quyền truy cập file.')
            }
          })

          // Use direct URL (or proxy if needed)
          audioPlayer.src = src
        }

        try {
          // Try to play
          await audioPlayer.play()
          isPlaying = true
          button.classList.remove('bg-primary')
          button.classList.add('bg-destructive')
          updateIcon(true)
        } catch (error) {
          console.error('Error playing audio:', error)
          isPlaying = false
          button.classList.remove('bg-destructive')
          button.classList.add('bg-primary')
          updateIcon(false)

          // Show error to user
          if (typeof window !== 'undefined') {
            alert('Không thể phát audio. Lỗi: ' + (error instanceof Error ? error.message : 'Unknown error'))
          }
        }
      }

      // Add click handler for the button
      button.addEventListener('click', (e) => {
        e.stopPropagation() // Prevent node selection in edit mode
        playAudio()
      })

      return {
        dom,
        contentDOM: null,
        destroy: () => {
          if (audioPlayer) {
            audioPlayer.pause()
            audioPlayer = null
          }
          button.removeEventListener('click', playAudio)
        },
      }
    }
  },
})
