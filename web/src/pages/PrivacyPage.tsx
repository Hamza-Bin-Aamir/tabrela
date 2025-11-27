export default function PrivacyPage() {
  return (
    <section>
      <h1 className="text-3xl font-bold">Privacy Policy</h1>
      <p className="mt-4 text-gray-700">
        Last updated: {new Date().toLocaleDateString()}
      </p>
      <p className="mt-4 text-gray-700">
        Your privacy is important to us. This policy outlines how we collect, use, and protect your information.
      </p>
      {/* Add privacy policy content here */}
    </section>
  )
}
