import { motion } from "motion/react"

export default function AvatarBox({ checked }: { checked?: boolean }) {
  return (
    <div className="relative mb-6 flex flex-col items-center justify-center">
      <div className="relative flex items-center justify-center rounded-2xl bg-white p-3 shadow-md border border-ui-border-base dark:bg-ui-bg-component">
        {checked && (
          <motion.div
            className="absolute -right-2 -top-2 flex size-6 items-center justify-center rounded-full border border-ui-border-base bg-[#3B82F6] text-white shadow-md"
            initial={{ opacity: 0, scale: 0.5 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{
              duration: 0.8,
              delay: 0.5,
              ease: [0, 0.71, 0.2, 1.01],
            }}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 20 20"
              fill="none"
            >
              <motion.path
                d="M5.8335 10.4167L9.16683 13.75L14.1668 6.25"
                stroke="white"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                initial={{ pathLength: 0, opacity: 0 }}
                animate={{ pathLength: 1, opacity: 1 }}
                transition={{
                  duration: 0.8,
                  delay: 0.8,
                  ease: [0.1, 0.8, 0.2, 1.01],
                }}
              />
            </svg>
          </motion.div>
        )}
        <img
          src="/stheno-logo.png"
          alt="Stheno by Syed Systems"
          className="h-16 w-auto max-w-[200px] object-contain"
        />
      </div>
    </div>
  )
}
