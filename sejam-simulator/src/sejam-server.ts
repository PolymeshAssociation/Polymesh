import express from 'express';
import cors from 'cors';
import bodyParser from 'body-parser';

const app = express();
const PORT = 3001;

// تنظیمات Middleware
app.use(cors()); // اجازه دسترسی از دامنه‌های دیگر (مثل localhost:3000 Next.js)
app.use(bodyParser.json()); // تجزیه JSON بدنه درخواست

// --- شبیه‌سازی دیتابیس سجام ---
// در محیط واقعی، این داده‌ها در پایگاه داده مرکزی سازمان بورس هستند.
// اینجا فقط چند رکورد تستی داریم.
const SEJAM_DB = [
  { nationalCode: '0012345678', sejamCode: 'SJ-TEST-001', fullName: 'Ali Rezaei' },
  { nationalCode: '0098765432', sejamCode: 'SJ-TEST-002', fullName: 'Sara Mohammadi' },
];

// --- Endpoint اصلی تأیید هویت ---
app.post('/api/sejam/verify', (req, res) => {
  const { nationalCode, sejamCode } = req.body;

  console.log(`🔍 Received verification request for National Code: ${nationalCode}`);

  // ۱. اعتبارسنجی اولیه ورودی‌ها
  if (!nationalCode || !sejamCode) {
    return res.status(400).json({
      status: 'error',
      message: 'National Code and Sejam Code are required.'
    });
  }

  // ۲. جستجو در دیتابیس شبیه‌سازی شده
  const userRecord = SEJAM_DB.find(
    u => u.nationalCode === nationalCode && u.sejamCode === sejamCode
  );

  // ۳. پاسخ‌دهی بر اساس نتیجه جستجو
  if (userRecord) {
    console.log(`✅ User Verified Successfully: ${userRecord.fullName}`);
    return res.json({
      status: 'success',
      data: {
        isVerified: true,
        fullName: userRecord.fullName,
        nationalCode: userRecord.nationalCode,
        timestamp: new Date().toISOString()
      }
    });
  } else {
    console.log(`❌ Verification Failed: Invalid combination.`);
    return res.status(401).json({
      status: 'error',
      message: 'Invalid National Code or Sejam Code.',
      isVerified: false
    });
  }
});

// شروع سرور
app.listen(PORT, () => {
  console.log(`🚀 Mock Sejam Server running on http://localhost:${PORT}`);
});