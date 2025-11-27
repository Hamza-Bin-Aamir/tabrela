import resend
import dotenv
import os

if(dotenv.load_dotenv(dotenv.find_dotenv())):

    resend.api_key = os.getenv("API_KEY")

    r = resend.Emails.send({
    "from": "baamir@email.giki-dt.com",
    "to": os.getenv("TEST_EMAIL"),
    "subject": "👀",
    "html": "<p><strong>yooo</strong></p>"
    })
