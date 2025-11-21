# Tabrela

> Tabbing in Reverse!

In traditional tabulation, the tab data lives on an island. Tabrela flips this model on its head. It's a **FOSS, debater-centric platform** designed from the ground up to be a permanent, portable, and verifiable "Debate Portfolio" for your entire debate career.

This project is not a replacement for tabbing software like Tabbycat. It's a **meta-layer** that sits on top of the circuit, allowing users to aggregate and verify their experience.

## 1. Core Features

Tabrela is being built as a tool that provides a direct, tangible product to every user: a single, verifiable "Debate Resume."

* **🗣️ Holistic Portfolios:** Track all your experience in one place: Speaking, Adjudication, Tabulation, CA, Org Comm, and Equity.
* **🎓 The "Learning Hub":** An opt-in library of mastery-based modules. New users can earn badges (e.g., `[Badge] Tabulation 101`) by passing automated, scenario-based simulators to prove their skills *before* they have experience.
* **📥 Portable & Downloadable:** Users own their data. Download your entire verified resume as a standardized `JSON` file or a printable `PDF` at any time.

## 2. Tech & Architecture Philosophy

This project is built to be a **non-profit public utility.**

* **FOSS & Self-Hostable:** This is the core principle. Tabrela is a protocol, not just one platform. Any institution can self-host its own instance.
* **Federated:** A user's cryptographically-signed data is designed to be portable, allowing migration between instances without losing credentials.
* **API-First:** The "Talent Discovery" engine is a set of API endpoints. The core Tabrela app is just the first client that consumes it.
* **Mobile-First & Accessible:** The platform *must* be lightweight, work on low-cost devices, and support translation (e.g., into Urdu) to be truly inclusive.

### Tech Stack

* **Frontend:** React
* **Backend:** Rust, Python
* **Database:** Postgres
* **Infra:** Docker + Microservices

## 3. Getting Started (Development)

1.  **Fork & Clone the repo:**
    ```bash
    git clone https://github.com/hamza-bin-aamir/tabrela.git
    cd tabrela
    ```
    or, if you want to use your own fork:

    ```bash
    git clone https://github.com/your-username/tabrela.git
    cd tabrela
    ```

2.  **Install dependencies:**
    ```bash
    npm install
    ```

3.  **Run the dev server:**
    ```bash
    # Example
    npm run dev
    ```

4.  For detailed setup, see `DOCKER.md` for our containerized environment.

## 4. How to Contribute

We are actively looking for contributors. This is a massive project, and we welcome help in all forms.

* **📖 Read our [CONTRIBUTING.md]** for setup guides and our code of conduct.
* **💬 Join the [Discussion on WhatsApp]** to be part of the design process.
* **🛠️ Check our [Project Roadmap/Issues]** to find a task.

## 5. License

This project is licensed under the **BSD License** - see the [LICENSE.md](LICENSE.md) file for details.