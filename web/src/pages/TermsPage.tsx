export default function TermsPage() {
  return (
    <section>
      <h1 className="text-3xl font-bold">Terms of Service</h1>
      <p className="mt-4 text-gray-700">
        Last updated: {new Date().toLocaleDateString()}
      </p>
      <p className="mt-4 text-gray-700">
        By using Tabrela, you agree to these terms and conditions.
      </p>
      {/* Add terms of service content here */}
    </section>
  )
}
