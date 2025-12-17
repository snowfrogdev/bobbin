# Code Smells and Anti-Patterns: Comprehensive Reference

*Extracted from Unit Testing, Clean Code, Code That Fits in Your Head, Refactoring, and Working Effectively with Legacy Code*

---

## Table of Contents

- [Naming Issues](#naming-issues)
- [Function Design](#function-design)
- [Class Design](#class-design)
- [Testing Anti-Patterns](#testing-anti-patterns)
- [Data and State Management](#data-and-state-management)
- [Conditional Logic](#conditional-logic)
- [Dependencies and Coupling](#dependencies-and-coupling)
- [Legacy Code Patterns](#legacy-code-patterns)
- [Comments](#comments)

---

## Naming Issues

### Mysterious Name

**Smell:** Names that don't clearly communicate purpose.

```javascript
// BAD
int d; // elapsed time in days
function x() { }
```

**Fix:** Use intention-revealing names.

```javascript
// GOOD
int elapsedTimeInDays;
function calculateTotalPrice() { }
```

**Source:** Clean Code, Refactoring

---

### Non-Pronounceable Names

**Smell:** Names that can't be spoken aloud.

```java
// BAD
class DtaRcrd102 {
    private Date genymdhms;
}
```

**Fix:** Use pronounceable names.

```java
// GOOD
class Customer {
    private Date generationTimestamp;
}
```

**Source:** Clean Code

---

### Unsearchable Names

**Smell:** Single-letter names and numeric constants.

```java
// BAD
for (int j=0; j<34; j++) {
    s += (t[j]*4)/5;
}
```

**Fix:** Use named constants and descriptive variables.

```java
// GOOD
int realDaysPerIdealDay = 4;
const int WORK_DAYS_PER_WEEK = 5;
int sum = 0;
for (int j=0; j < NUMBER_OF_TASKS; j++) {
    int realTaskDays = taskEstimate[j] * realDaysPerIdealDay;
    int realTaskWeeks = (realTaskDays / WORK_DAYS_PER_WEEK);
    sum += realTaskWeeks;
}
```

**Source:** Clean Code

---

### Noise Words

**Smell:** Names with meaningless distinctions that don't convey intent.

```java
// BAD - noise words that don't distinguish meaning
public static void copyChars(char a1[], char a2[]) {
    for (int i = 0; i < a1.length; i++) {
        a2[i] = a1[i];
    }
}
```

**Fix:** Make meaningful distinctions that reveal intent.

```java
// GOOD - clear, meaningful names
public static void copyChars(char source[], char destination[]) {
    for (int i = 0; i < source.length; i++) {
        destination[i] = source[i];
    }
}
```

**Source:** Clean Code

---

### Disinformation

**Smell:** Names that mislead about the actual type or behavior.

```java
// BAD - implies it's a List, but might be an array or other collection
Account[] accountList;
Map<String, Account> accountList;

// Also bad - varying names in small ways
XYZControllerForEfficientHandlingOfStrings
XYZControllerForEfficientStorageOfStrings
```

**Fix:** Use accurate names that don't imply false information.

```java
// GOOD - accurate to the type
Account[] accounts;
Map<String, Account> accountMap;
List<Account> accountList;  // Only if it's actually a List

// Better - clearly distinct names
StringHandler
StringRepository
```

**Source:** Clean Code

---

### Hungarian Notation / Encodings

**Smell:** Prefixes that encode type or scope.

```csharp
// BAD
private int m_age;
IShapeFactory factory;
```

**Fix:** Avoid encodings; let the type system do its job.

```csharp
// GOOD
private int age;
ShapeFactory factory;
```

**Source:** Clean Code, Code That Fits in Your Head

---

## Function Design

### Long Function

**Smell:** Functions longer than one screen, or needing comments to explain sections.

```javascript
// BAD
function statement(invoice, plays) {
  let totalAmount = 0;
  let volumeCredits = 0;
  let result = `Statement for ${invoice.customer}\n`;
  // ... 50+ lines of code
}
```

**Fix:** Extract Function (99% of the time).

```javascript
// GOOD
function statement(invoice, plays) {
  return renderPlainText(createStatementData(invoice, plays));
}

function renderPlainText(data) {
  let result = `Statement for ${data.customer}\n`;
  // ... rendering logic
  return result;
}
```

**Additional Techniques:**
- Replace Temp with Query
- Decompose Conditional
- Replace Conditional with Polymorphism
- Split Loop

**Source:** Clean Code, Refactoring, Code That Fits in Your Head

**Guideline:** Keep functions ≤ 24 lines (80/24 rule) and cyclomatic complexity ≤ 7.

---

### Long Parameter List

**Smell:** Functions with many parameters (3+).

```java
// BAD
public void create(String name, String email, DateTime at, int quantity) { }
```

**Fix:** Introduce Parameter Object or Preserve Whole Object.

```java
// GOOD - Parameter Object
public void create(ReservationRequest request) { }

public class ReservationRequest {
    private String name;
    private String email;
    private DateTime at;
    private int quantity;
}

// GOOD - Preserve Whole Object
public void withinRange(NumberRange range) {
    return range.low >= this.min && range.high <= this.max;
}
```

**Source:** Clean Code, Refactoring, Code That Fits in Your Head

---

### Flag Arguments

**Smell:** Boolean flags that make function do more than one thing.

```java
// BAD
render(boolean isSuite)

public int calculateWeeklyPay(boolean overtime) {
    if (overtime) { /* ... */ }
    else { /* ... */ }
}
```

**Fix:** Split into separate functions.

```java
// GOOD
renderForSuite()
renderForSingleTest()

public int straightPay() { /* ... */ }
public int overTimePay() { /* ... */ }
```

**Source:** Clean Code, Refactoring

---

### Side Effects

**Smell:** Function that does more than its name suggests.

```java
// BAD
public boolean checkPassword(String userName, String password) {
    // ... password checking logic
    Session.initialize(); // HIDDEN SIDE EFFECT!
    return true;
}
```

**Fix:** Either rename to reveal the side effect or separate the operations.

```java
// OPTION 1: Rename
public boolean checkPasswordAndInitializeSession(String userName, String password) {
    // Now the name reveals what it does
}

// OPTION 2: Separate (BETTER)
public boolean checkPassword(String userName, String password) {
    // Only check password
}

public void initializeSession() {
    // Separate responsibility
}
```

**Source:** Clean Code

---

### Command-Query Separation Violation

**Smell:** Function both returns a value and has side effects.

```java
// BAD
public boolean set(String attribute, String value); // Does and returns

// Confusing usage:
if (set("username", "unclebob"))...
```

**Fix:** Separate into query and command.

```java
// GOOD
if (attributeExists("username")) {    // Query
    setAttribute("username", "unclebob");  // Command
}
```

**Source:** Clean Code, Code That Fits in Your Head

---

### Multiple Levels of Abstraction

**Smell:** Mixing high-level and low-level operations.

```java
// BAD
public void render(PageData pageData) {
    response.setContentType("text/html");  // High level
    String pagePathName = PathParser.render(pagePath);  // Mid level
    response.write("<html><body>...");  // Low level
}
```

**Fix:** Keep consistent abstraction level per function.

```java
// GOOD
public void render(PageData pageData) {
    response.setContentType("text/html");
    renderPageContent(pageData);
}

private void renderPageContent(PageData pageData) {
    // All code at same abstraction level
}
```

**Source:** Clean Code, Code That Fits in Your Head

---

### Cyclomatic Complexity > 7

**Smell:** Too many pathways through code (if, for, while, switch, etc.).

```csharp
// BAD - Complexity = 7
public async Task<ActionResult> Post(ReservationDto dto) {
    if (dto is null) throw new ArgumentNullException(nameof(dto));  // 1
    if (!DateTime.TryParse(dto.At, out var d)) return BadRequest();  // 2
    if (dto.Email is null) return BadRequest();  // 3
    if (dto.Quantity < 1) return BadRequest();  // 4

    var reservations = await Repository.ReadReservations(d);
    int reservedSeats = reservations.Sum(r => r.Quantity);
    if (10 < reservedSeats + dto.Quantity) return Error();  // 5

    var r = new Reservation(d, dto.Email, dto.Name ?? "", dto.Quantity);  // 6 (??)
    await Repository.Create(r);
    return new NoContentResult();
}
```

**Fix:** Extract methods to reduce complexity.

```csharp
// GOOD
public async Task<ActionResult> Post(ReservationDto dto) {
    Reservation? reservation = dto.Validate(id);
    if (reservation is null) return BadRequest();

    bool accepted = await MaitreD.WillAccept(reservation);
    if (!accepted) return Error();

    await Repository.Create(reservation);
    return NoContent();
}

private static bool IsValid(ReservationDto dto) {
    return DateTime.TryParse(dto.At, out _)
        && !(dto.Email is null)
        && 0 < dto.Quantity;
}
```

**Source:** Code That Fits in Your Head

**Guideline:** Keep cyclomatic complexity ≤ 7 (matches human working memory).

---

### Variables in Scope > 7

**Smell:** Too many things to track at once (parameters + locals + fields).

```csharp
// BAD
public void Process(int a, int b, int c, int d) {  // 4 parameters
    var x = a + b;     // 1 local
    var y = x * 2;     // 2 locals
    var z = y + field1; // 3 locals + 1 field = 4 things
    var w = z + field2; // 4 locals + 2 fields = 6 things
    var v = w + c;     // 5 locals + 2 fields + params = 9+ things to track!
}
```

**Fix:** Extract helper methods to reduce scope.

```csharp
// GOOD
public void Process(int a, int b, int c, int d) {
    var intermediate = Calculate(a, b);  // Hides details
    UpdateFields(intermediate, c, d);
}

private int Calculate(int a, int b) {  // Smaller scope
    var sum = a + b;
    return sum * 2;
}
```

**Source:** Code That Fits in Your Head

**Guideline:** Keep total items in scope ≤ 7.

---

### Functions Doing More Than One Thing

**Smell:** Function has multiple responsibilities.

```java
// BAD - does three things
public void pay() {
    for (Employee e : employees) {  // 1. Loops over employees
        if (e.isPayday()) {  // 2. Checks if each should be paid
            Money pay = e.calculatePay();
            e.deliverPay(pay);  // 3. Pays them
        }
    }
}
```

**Fix:** Split into single-purpose functions.

```java
// GOOD - each does one thing
public void pay() {
    for (Employee e : employees)
        payIfNecessary(e);
}

private void payIfNecessary(Employee e) {
    if (e.isPayday())
        calculateAndDeliverPay(e);
}

private void calculateAndDeliverPay(Employee e) {
    Money pay = e.calculatePay();
    e.deliverPay(pay);
}
```

**Source:** Clean Code, Refactoring

---

## Class Design

### Large Class

**Smell:** Too many fields, too much code, too many responsibilities.

```java
// BAD
public class SuperDashboard extends JFrame {
    public Component getLastFocusedComponent()
    public void setLastFocused(Component lastFocused)
    public int getMajorVersionNumber()
    public int getMinorVersionNumber()
    public int getBuildNumber()
    // ... 50 more methods
}
```

**Fix:** Extract Class based on cohesion.

```java
// GOOD
public class Version {
    public int getMajorVersionNumber()
    public int getMinorVersionNumber()
    public int getBuildNumber()
}

public class SuperDashboard extends JFrame {
    private Version version;
    public Component getLastFocusedComponent()
    public void setLastFocused(Component lastFocused)
    public Version getVersion() { return version; }
}
```

**Source:** Clean Code, Refactoring

---

### Low Cohesion

**Smell:** Methods that don't use the same fields.

```csharp
// BAD - Low cohesion
public class CustomerProcessor {
    private Database db;
    private EmailService email;
    private Logger logger;

    public void saveCustomer() {
        // Only uses db
    }

    public void sendWelcomeEmail() {
        // Only uses email
    }

    public void logActivity() {
        // Only uses logger
    }
}
```

**Fix:** Split into cohesive classes.

```csharp
// GOOD
public class CustomerRepository {
    private Database db;
    public void saveCustomer() { }
}

public class CustomerNotifier {
    private EmailService email;
    public void sendWelcomeEmail() { }
}

public class ActivityLogger {
    private Logger logger;
    public void logActivity() { }
}
```

**Source:** Clean Code, Code That Fits in Your Head

**Principle:** "Things that change at the same rate belong together."

---

### Feature Envy

**Smell:** Method more interested in another class than its own.

```csharp
// BAD
public class ShoppingCart {
    public double calculateTotal() {
        double total = 0;
        for (Item item : items) {
            total += item.getPrice() * item.getQuantity();  // Envies Item
            total -= item.getDiscount();  // Envies Item
        }
        return total;
    }
}
```

**Fix:** Move method or use Tell, Don't Ask.

```csharp
// GOOD
public class Item {
    public double getSubtotal() {
        return (price * quantity) - discount;
    }
}

public class ShoppingCart {
    public double calculateTotal() {
        return items.stream()
            .mapToDouble(Item::getSubtotal)
            .sum();
    }
}
```

**Source:** Clean Code, Refactoring, Code That Fits in Your Head

---

### Data Class

**Smell:** Classes with only fields and getters/setters, no behavior.

```java
// BAD
public class Customer {
    public String name;
    public String email;
    public int age;

    // Only getters and setters
}
```

**Fix:** Move behavior into the data class.

```java
// GOOD
public class Customer {
    private String name;
    private String email;
    private int age;

    public boolean isAdult() {
        return age >= 18;
    }

    public void sendEmail(String message) {
        // Behavior with data
    }
}
```

**Note:** Immutable DTOs/records from Split Phase are acceptable exceptions.

**Source:** Clean Code, Refactoring

---

### Refused Bequest

**Smell:** Subclass doesn't want inherited methods/data.

```java
// BAD - Stack refuses List methods
public class Stack extends ArrayList {
    public void push(Object o) { add(o); }
    public Object pop() { return remove(size() - 1); }
    // But also inherits get(), set(), etc. that break Stack semantics
}
```

**Fix:** Use composition instead of inheritance.

```java
// GOOD
public class Stack {
    private List<Object> elements = new ArrayList<>();

    public void push(Object o) { elements.add(o); }
    public Object pop() { return elements.remove(elements.size() - 1); }
    // Only expose Stack operations
}
```

**Alternative:** Replace Subclass with Delegate or Replace Superclass with Delegate.

**Source:** Clean Code, Refactoring

---

### Alternative Classes with Different Interfaces

**Smell:** Classes that do similar things with different interfaces.

```java
// BAD
public class EmailNotifier {
    public void sendEmail(String message) { }
}

public class SMSNotifier {
    public void transmitSMS(String text) { }  // Different interface
}
```

**Fix:** Unify interfaces through Extract Superclass or Change Function Declaration.

```java
// GOOD
public interface Notifier {
    void send(String message);
}

public class EmailNotifier implements Notifier {
    public void send(String message) { /* email logic */ }
}

public class SMSNotifier implements Notifier {
    public void send(String message) { /* SMS logic */ }
}
```

**Source:** Refactoring

---

### Middle Man

**Smell:** Half a class's methods just delegate to another class.

```java
// BAD
public class Person {
    private Department department;

    public String getManagerName() {
        return department.getManager().getName();
    }

    public String getDepartmentBudget() {
        return department.getBudget();
    }

    // 10 more delegating methods...
}
```

**Fix:** Remove Middle Man and access object directly.

```java
// GOOD
public class Person {
    private Department department;

    public Department getDepartment() {
        return department;
    }
}

// Client:
String managerName = person.getDepartment().getManager().getName();
```

**Balance:** Some delegation is good (encapsulation), but too much is irritating.

**Source:** Refactoring

---

## Testing Anti-Patterns

### Testing Private Methods

**Smell:** Making private methods public just to test them.

```csharp
// BAD
public class Order {
    public decimal GetPrice() { ... }  // Made public for testing
}
```

**Fix:** Test through public API or extract to separate class.

```csharp
// OPTION 1: Test through public API
[Fact]
public void Order_description_includes_price() {
    var order = new Order();
    string description = order.GenerateDescription();
    Assert.Contains("price: 100", description);
}

// OPTION 2: Extract to separate class
public class PriceCalculator {
    public decimal Calculate(...) { ... }
}
```

**Source:** Unit Testing, Clean Code

---

### Exposing Private State for Testing

**Smell:** Making fields public or adding getters just for tests.

```csharp
// BAD
public class Customer {
    public CustomerStatus Status { get; set; } // Made public for testing
}
```

**Fix:** Test observable behavior instead.

```csharp
// GOOD
public class Customer {
    private CustomerStatus _status = CustomerStatus.Regular;

    public void Promote() {
        _status = CustomerStatus.Preferred;
    }

    public decimal GetDiscount() {
        return _status == CustomerStatus.Preferred ? 0.05m : 0m;
    }
}

[Fact]
public void Promoted_customer_gets_discount() {
    var customer = new Customer();
    customer.Promote();

    decimal discount = customer.GetDiscount();
    Assert.Equal(0.05m, discount);
}
```

**Source:** Unit Testing

---

### Leaking Domain Knowledge to Tests

**Smell:** Test duplicates production algorithm.

```csharp
// BAD - Test duplicates logic
[Theory]
[InlineData(1, 3)]
[InlineData(11, 33)]
public void Adding_two_numbers(int value1, int value2) {
    int expected = value1 + value2; // DUPLICATES ALGORITHM!

    int actual = Calculator.Add(value1, value2);

    Assert.Equal(expected, actual);
}
```

**Fix:** Hard-code expected results.

```csharp
// GOOD
[Theory]
[InlineData(1, 3, 4)]
[InlineData(11, 33, 44)]
[InlineData(100, 500, 600)]
public void Adding_two_numbers(int value1, int value2, int expected) {
    int actual = Calculator.Add(value1, value2);
    Assert.Equal(expected, actual);
}
```

**Source:** Unit Testing

---

### Code Pollution

**Smell:** Production code contains test-specific logic.

```csharp
// BAD
public class Logger {
    private readonly bool _isTestEnvironment;

    public Logger(bool isTestEnvironment) {
        _isTestEnvironment = isTestEnvironment;
    }

    public void Log(string message) {
        if (_isTestEnvironment)
            return; // Don't log in tests - POLLUTION!

        // Log to file
    }
}
```

**Fix:** Use interfaces and dependency injection.

```csharp
// GOOD
public interface ILogger {
    void Log(string message);
}

public class Logger : ILogger {
    public void Log(string message) {
        // Log to file
    }
}

public class FakeLogger : ILogger {
    public void Log(string message) {
        // Do nothing or record
    }
}
```

**Source:** Unit Testing

---

### Over-Mocking

**Smell:** Everything is mocked, including domain objects.

```csharp
// BAD
[Fact]
public void Process_order() {
    var productMock = new Mock<IProduct>();
    productMock.Setup(x => x.GetPrice()).Returns(10);

    var customerMock = new Mock<ICustomer>();
    customerMock.Setup(x => x.GetDiscount()).Returns(0.05m);

    var orderLineMock = new Mock<IOrderLine>();
    // ... lots more mocking
}
```

**Fix:** Use real objects, only mock external dependencies.

```csharp
// GOOD
[Fact]
public void Process_order() {
    var customer = new Customer { Type = CustomerType.Preferred };
    var product = new Product { Price = 10 };
    var order = new Order {
        Customer = customer,
        Lines = new List<OrderLine> {
            new OrderLine { Product = product, Quantity = 3 }
        }
    };

    var sut = new OrderProcessor();
    decimal total = sut.CalculateTotal(order);

    Assert.Equal(28.5m, total);
}
```

**Source:** Unit Testing, Code That Fits in Your Head

---

### Testing Implementation Instead of Behavior

**Smell:** Tests verify internal implementation details.

```csharp
// BAD
[Fact]
public void Validates_using_all_validators() {
    var validator1 = new Mock<IOrderValidator>();
    var validator2 = new Mock<IOrderValidator>();

    var sut = new OrderProcessor(validator1.Object, validator2.Object);
    var order = new Order();

    sut.Validate(order);

    // Testing HOW it validates, not WHAT happens
    validator1.Verify(x => x.Validate(order), Times.Once);
    validator2.Verify(x => x.Validate(order), Times.Once);
}
```

**Fix:** Test observable behavior.

```csharp
// GOOD
[Fact]
public void Invalid_order_fails_validation() {
    var order = new OrderBuilder()
        .WithProduct(null) // Invalid!
        .Build();

    var sut = new OrderProcessor();
    var result = sut.Validate(order);

    Assert.False(result.IsValid);
    Assert.Contains("product", result.ErrorMessage.ToLower());
}
```

**Source:** Unit Testing, Clean Code

---

### Asserting on Stubs

**Smell:** Verifying interactions with stubs (input providers).

```csharp
// BAD
var stub = new Mock<IStore>();
stub.Setup(x => x.HasEnoughInventory(Product.Shampoo, 5))
    .Returns(true);

customer.Purchase(stub.Object, Product.Shampoo, 5);

stub.Verify(x => x.HasEnoughInventory(Product.Shampoo, 5)); // DON'T!
```

**Fix:** Only verify mocks (outgoing commands), not stubs.

```csharp
// GOOD
var storeMock = new Mock<IStore>();
storeMock.Setup(x => x.HasEnoughInventory(Product.Shampoo, 5))
    .Returns(true);

customer.Purchase(storeMock.Object, Product.Shampoo, 5);

// Verify state change or outgoing command
Assert.Equal(1, customer.PurchaseCount);
```

**Principle:** Mocks verify, stubs provide. Never assert on stubs.

**Source:** Unit Testing

---

### Multiple AAA Sections in One Test

**Smell:** Multiple Act phases indicate testing multiple concepts.

```csharp
// BAD
[Fact]
public void Purchase_two_items() {
    // Arrange
    var store = new Store();
    store.AddInventory(Product.Shampoo, 10);
    var customer = new Customer();

    // Act 1
    bool success1 = customer.Purchase(store, Product.Shampoo, 5);
    // Assert 1
    Assert.True(success1);

    // Act 2 - SECOND ACT IS A SMELL
    bool success2 = customer.Purchase(store, Product.Shampoo, 5);
    // Assert 2
    Assert.True(success2);
}
```

**Fix:** Split into separate tests.

```csharp
// GOOD
[Fact]
public void First_purchase_succeeds_when_inventory_available() {
    var store = new Store();
    store.AddInventory(Product.Shampoo, 10);
    var customer = new Customer();

    bool success = customer.Purchase(store, Product.Shampoo, 5);

    Assert.True(success);
}

[Fact]
public void Second_purchase_succeeds_when_inventory_still_available() {
    var store = new Store();
    store.AddInventory(Product.Shampoo, 10);
    var customer = new Customer();
    customer.Purchase(store, Product.Shampoo, 5); // First purchase

    bool success = customer.Purchase(store, Product.Shampoo, 5);

    Assert.True(success);
}
```

**Source:** Unit Testing, Code That Fits in Your Head

---

### If Statements in Tests

**Smell:** Conditional logic in tests.

```csharp
// BAD
[Fact]
public void Purchase_succeeds() {
    bool success = customer.Purchase(store, Product.Shampoo, 5);

    if (success)
        Assert.Equal(5, store.GetInventory(Product.Shampoo));
}
```

**Fix:** Tests should be linear, no branching.

```csharp
// GOOD
[Fact]
public void Successful_purchase_reduces_inventory() {
    customer.Purchase(store, Product.Shampoo, 5);

    Assert.Equal(5, store.GetInventory(Product.Shampoo));
}
```

**Source:** Unit Testing

---

### Reusing Database Contexts

**Smell:** Sharing database context across test phases.

```csharp
// BAD
using (var context = new CrmContext(ConnectionString)) {
    // Arrange
    var user = new User(...);
    context.Users.Add(user);
    context.SaveChanges();

    var sut = new UserController(context); // SAME CONTEXT!

    // Act
    sut.ChangeEmail(user.Id, "new@email.com");

    // Assert
    var userFromDb = context.Users.Find(user.Id); // SAME CONTEXT!
}
```

**Fix:** Use separate contexts for each phase.

```csharp
// GOOD
using (var context = new CrmContext(ConnectionString)) {
    // Arrange - context 1
    var user = new User(...);
    context.Users.Add(user);
    context.SaveChanges();
}

using (var context = new CrmContext(ConnectionString)) {
    // Act - context 2
    var sut = new UserController(context);
    sut.ChangeEmail(user.Id, "new@email.com");
}

using (var context = new CrmContext(ConnectionString)) {
    // Assert - context 3
    var userFromDb = context.Users.Find(user.Id);
    Assert.Equal("new@email.com", userFromDb.Email);
}
```

**Source:** Unit Testing

---

### Time as Ambient Context

**Smell:** Using DateTime.Now or other ambient context in production code.

```csharp
// BAD
public static class DateTimeServer {
    private static Func<DateTime> _func;
    public static DateTime Now => _func();

    public static void Init(Func<DateTime> func) {
        _func = func;
    }
}

// Test
DateTimeServer.Init(() => new DateTime(2020, 1, 1));
```

**Fix:** Inject time as dependency.

```csharp
// GOOD
public interface IDateTimeServer {
    DateTime Now { get; }
}

public class InquiryController {
    private readonly IDateTimeServer _dateTimeServer;

    public InquiryController(IDateTimeServer dateTimeServer) {
        _dateTimeServer = dateTimeServer;
    }

    public void ApproveInquiry(int id) {
        var inquiry = GetById(id);
        inquiry.Approve(_dateTimeServer.Now);
        SaveInquiry(inquiry);
    }
}

// Test
var dateTimeStub = new Mock<IDateTimeServer>();
dateTimeStub.Setup(x => x.Now).Returns(new DateTime(2020, 1, 1));
```

**Source:** Unit Testing, Code That Fits in Your Head

---

## Data and State Management

### Global Data

**Smell:** Global variables, class variables, singletons accessible from anywhere.

```java
// BAD
public class Settings {
    public static int MAX_CONNECTIONS = 10;
}

// Anyone can modify this anywhere
Settings.MAX_CONNECTIONS = 100;
```

**Fix:** Encapsulate Variable.

```java
// GOOD
public class Settings {
    private static int maxConnections = 10;

    public static int getMaxConnections() {
        return maxConnections;
    }

    public static void setMaxConnections(int value) {
        if (value < 1 || value > 100)
            throw new IllegalArgumentException("Must be 1-100");
        maxConnections = value;
    }
}
```

**Source:** Clean Code, Refactoring

**Principle:** "The difference between a poison and something benign is the dose."

---

### Mutable Data

**Smell:** Data that changes causing unexpected consequences.

```java
// BAD
public class Account {
    public double balance;  // Mutable, public
}

// Anywhere in code:
account.balance = -1000;  // Oops!
```

**Fix:** Encapsulate, use immutability where possible.

```java
// GOOD
public class Account {
    private double balance;

    public double getBalance() {
        return balance;
    }

    public void deposit(double amount) {
        if (amount <= 0)
            throw new IllegalArgumentException("Amount must be positive");
        balance += amount;
    }
}
```

**Source:** Clean Code, Refactoring, Code That Fits in Your Head

---

### Primitive Obsession

**Smell:** Using primitives (int, string) for domain concepts.

```java
// BAD
public void processPayment(double amount, String currency) {
    // Money represented as primitives
}

String phoneNumber = "555-1234";  // String for phone
int zipCode = 12345;  // int for zip
```

**Fix:** Create domain-specific types.

```java
// GOOD
public class Money {
    private final double amount;
    private final Currency currency;

    public Money(double amount, Currency currency) {
        if (amount < 0)
            throw new IllegalArgumentException("Amount cannot be negative");
        this.amount = amount;
        this.currency = currency;
    }

    // Domain operations
    public Money add(Money other) {
        if (!this.currency.equals(other.currency))
            throw new IllegalArgumentException("Currency mismatch");
        return new Money(this.amount + other.amount, this.currency);
    }
}

public class PhoneNumber {
    private final String value;

    public PhoneNumber(String value) {
        if (!isValid(value))
            throw new IllegalArgumentException("Invalid phone number");
        this.value = normalize(value);
    }
}

// Usage
public void processPayment(Money amount) {
    // Type-safe, enforces invariants
}
```

**Source:** Clean Code, Refactoring, Code That Fits in Your Head

---

### Data Clumps

**Smell:** Same group of data items appearing together everywhere.

```java
// BAD
public void createWindow(int x, int y, int width, int height) { }
public void moveWindow(int x, int y) { }
public void resizeWindow(int width, int height) { }
```

**Fix:** Extract Class or Introduce Parameter Object.

```java
// GOOD
public class Point {
    private final int x;
    private final int y;

    public Point(int x, int y) {
        this.x = x;
        this.y = y;
    }
}

public class Dimension {
    private final int width;
    private final int height;

    public Dimension(int width, int height) {
        this.width = width;
        this.height = height;
    }
}

public void createWindow(Point position, Dimension size) { }
public void moveWindow(Point position) { }
public void resizeWindow(Dimension size) { }
```

**Source:** Clean Code, Refactoring

---

### Temporary Field

**Smell:** Field set only in certain circumstances.

```java
// BAD
public class Order {
    private double discountAmount;  // Only used during discount calculation

    public double calculateTotal() {
        double base = getBasePrice();
        if (hasDiscount()) {
            discountAmount = calculateDiscount();  // Set here
            return base - discountAmount;
        }
        return base;
    }
}
```

**Fix:** Extract Class for the temporary state.

```java
// GOOD
public class DiscountCalculation {
    private final double basePrice;
    private final double discountAmount;

    public DiscountCalculation(double basePrice, Discount discount) {
        this.basePrice = basePrice;
        this.discountAmount = discount.calculate(basePrice);
    }

    public double getTotal() {
        return basePrice - discountAmount;
    }
}
```

**Source:** Refactoring

---

### Validate, Don't Parse

**Smell:** Validation scattered; downstream code doesn't know if validated.

```csharp
// BAD
if (!DateTime.TryParse(dto.At, out var d))
    return new BadRequestResult();
// Later: Is dto.Email validated? Who knows?
```

**Fix:** Parse into domain type that represents validity.

```csharp
// GOOD
Reservation? r = dto.Validate(id);  // Returns null or valid Reservation
if (r is null)
    return new BadRequestResult();
// Later: r is guaranteed valid - type system enforces it

// Or use types
public class Email {
    private readonly string value;

    public Email(string email) {
        if (!IsValid(email))
            throw new ArgumentException("Invalid email");
        value = email;
    }

    private static bool IsValid(string email) =>
        email.Contains("@");

    public override string ToString() => value;
}

// Now you can't create invalid emails
```

**Principle:** Make invalid states unrepresentable.

**Source:** Code That Fits in Your Head

---

## Conditional Logic

### Repeated Switches / Type Codes

**Smell:** Same switch/case or cascading if/else appearing in multiple places.

```javascript
// BAD - Switch appears in multiple methods
function plumage(bird) {
    switch (bird.type) {
        case 'EuropeanSwallow':
            return "average";
        case 'AfricanSwallow':
            return (bird.numberOfCoconuts > 2) ? "tired" : "average";
        case 'NorwegianBlueParrot':
            return (bird.voltage > 100) ? "scorched" : "beautiful";
    }
}

function airSpeedVelocity(bird) {
    switch (bird.type) {  // DUPLICATE SWITCH!
        case 'EuropeanSwallow':
            return 35;
        case 'AfricanSwallow':
            return 40 - 2 * bird.numberOfCoconuts;
        case 'NorwegianBlueParrot':
            return (bird.voltage > 100) ? 0 : 10 + bird.voltage / 10;
    }
}
```

**Fix:** Replace Conditional with Polymorphism.

```javascript
// GOOD - One factory switch, then polymorphism everywhere
function createBird(bird) {
    switch (bird.type) {  // ONLY switch statement
        case 'EuropeanSwallow':
            return new EuropeanSwallow(bird);
        case 'AfricanSwallow':
            return new AfricanSwallow(bird);
        case 'NorwegianBlueParrot':
            return new NorwegianBlueParrot(bird);
    }
}

class Bird {
    get plumage() { return "unknown"; }
    get airSpeedVelocity() { return 0; }
}

class EuropeanSwallow extends Bird {
    get plumage() { return "average"; }
    get airSpeedVelocity() { return 35; }
}

class AfricanSwallow extends Bird {
    get plumage() {
        return (this.numberOfCoconuts > 2) ? "tired" : "average";
    }
    get airSpeedVelocity() {
        return 40 - 2 * this.numberOfCoconuts;
    }
}

// Usage - no switches needed
let bird = createBird(birdData);
console.log(bird.plumage());  // Polymorphic
console.log(bird.airSpeedVelocity());  // Polymorphic
```

**Rule:** "ONE SWITCH" - There may be no more than one switch statement for a given type selection.

**Source:** Clean Code, Refactoring

---

### Complex Conditionals

**Smell:** Nested or complex conditional expressions.

```java
// BAD
if (!aDate.isBefore(plan.summerStart) && !aDate.isAfter(plan.summerEnd))
    charge = quantity * plan.summerRate;
else
    charge = quantity * plan.regularRate + plan.regularServiceCharge;
```

**Fix:** Decompose Conditional - extract condition and branches.

```java
// GOOD
if (summer())
    charge = summerCharge();
else
    charge = regularCharge();

private boolean summer() {
    return !aDate.isBefore(plan.summerStart) && !aDate.isAfter(plan.summerEnd);
}

private double summerCharge() {
    return quantity * plan.summerRate;
}

private double regularCharge() {
    return quantity * plan.regularRate + plan.regularServiceCharge;
}
```

**Even Better:** Use ternary for simple cases.

```java
charge = summer() ? summerCharge() : regularCharge();
```

**Source:** Clean Code, Refactoring

---

### Nested Conditionals

**Smell:** Deep nesting makes code hard to follow.

```java
// BAD
public void payAmount(Employee employee) {
    if (employee.isSeparated) {
        result = {amount: 0, reasonCode: "SEP"};
    }
    else {
        if (employee.isRetired) {
            result = {amount: 0, reasonCode: "RET"};
        }
        else {
            // 20 lines of normal payment logic
        }
    }
    return result;
}
```

**Fix:** Replace Nested Conditional with Guard Clauses.

```java
// GOOD
public void payAmount(Employee employee) {
    if (employee.isSeparated) return {amount: 0, reasonCode: "SEP"};
    if (employee.isRetired) return {amount: 0, reasonCode: "RET"};

    // Normal payment logic at main level
    return someFinalComputation();
}
```

**Principle:** Guard clauses make special cases obvious; use them when one branch is exceptional.

**Source:** Clean Code, Refactoring

---

### Consolidate Conditional Expression

**Smell:** Separate checks with same result.

```java
// BAD
function disabilityAmount(anEmployee) {
    if (anEmployee.seniority < 2) return 0;
    if (anEmployee.monthsDisabled > 12) return 0;
    if (anEmployee.isPartTime) return 0;
    // compute the disability amount
}
```

**Fix:** Combine with logical operators, then extract.

```java
// GOOD
function disabilityAmount(anEmployee) {
    if (isNotEligibleForDisability()) return 0;
    // compute the disability amount

    function isNotEligibleForDisability() {
        return ((anEmployee.seniority < 2)
                || (anEmployee.monthsDisabled > 12)
                || (anEmployee.isPartTime));
    }
}
```

**Source:** Refactoring

---

### Encapsulate Conditionals

**Smell:** Complex inline conditionals.

```java
// BAD
if (timer.hasExpired() && !timer.isRecurrent())

if (!buffer.shouldNotCompact())  // Negative
```

**Fix:** Extract to well-named method.

```java
// GOOD
if (shouldBeDeleted(timer))

if (buffer.shouldCompact())  // Positive
```

**Source:** Clean Code, Refactoring

---

### Boundary Conditions Scattered

**Smell:** Boundary calculations duplicated throughout code.

```java
// BAD
if (level + 1 < tags.length) {
    parts = new Parse(body, tags, level + 1, offset + endTag);
    body = null;
}
// Elsewhere: level + 1 appears again
```

**Fix:** Encapsulate boundary condition.

```java
// GOOD
int nextLevel = level + 1;
if (nextLevel < tags.length) {
    parts = new Parse(body, tags, nextLevel, offset + endTag);
    body = null;
}
```

**Source:** Clean Code, Code That Fits in Your Head

---

## Dependencies and Coupling

### Message Chains / Train Wrecks

**Smell:** Client navigating through series of objects.

```java
// BAD
String outputDir = ctxt.getOptions().getScratchDir().getAbsolutePath();

manager = aPerson.department.manager;  // Knows traversal structure
```

**Fix:** Hide Delegate - add methods to hide the chain.

```java
// GOOD
String outputDir = ctxt.getOutputDirectory();

// In Context class:
public String getOutputDirectory() {
    return options.getScratchDir().getAbsolutePath();
}

// Person class:
public Person getManager() {
    return department.manager;
}

// Usage:
manager = aPerson.getManager();
```

**Alternative:** Extract Function + Move Function to move usage down chain.

**Source:** Clean Code, Refactoring

**Principle:** Law of Demeter - "Talk to friends, not to strangers."

---

### Insider Trading

**Smell:** Modules excessively exchanging data behind the scenes.

```java
// BAD - Subclasses know too much about parent internals
public class BaseProcessor {
    protected List<Item> items;
    protected int processingStage;
}

public class SpecialProcessor extends BaseProcessor {
    public void process() {
        // Directly manipulates parent's items and processingStage
        items.clear();
        processingStage = 2;
    }
}
```

**Fix:** Move Function/Field or Replace Subclass with Delegate.

```java
// GOOD
public class BaseProcessor {
    private List<Item> items;
    private int processingStage;

    protected void resetItems() {
        items.clear();
    }

    protected void setStage(int stage) {
        processingStage = stage;
    }
}

public class SpecialProcessor extends BaseProcessor {
    public void process() {
        resetItems();
        setStage(2);
    }
}
```

**Source:** Refactoring

---

### Divergent Change

**Smell:** One module changed for different reasons.

```java
// BAD
public class CustomerService {
    public void updateCustomer() {
        // Database logic
        db.save(customer);

        // Notification logic
        email.send(customer);

        // Validation logic
        if (!validator.isValid(customer))
            throw new Exception();
    }
}
```

**Fix:** Split Phase or Extract Class.

```java
// GOOD
public class CustomerRepository {
    public void save(Customer customer) {
        db.save(customer);
    }
}

public class CustomerNotifier {
    public void sendUpdateEmail(Customer customer) {
        email.send(customer);
    }
}

public class CustomerValidator {
    public boolean isValid(Customer customer) {
        // Validation logic
    }
}

public class CustomerService {
    public void updateCustomer() {
        validator.validate(customer);
        repository.save(customer);
        notifier.sendUpdateEmail(customer);
    }
}
```

**Principle:** "One module should have one reason to change."

**Source:** Clean Code, Refactoring

---

### Shotgun Surgery

**Smell:** Every change requires editing lots of different classes.

```java
// BAD - Adding a new discount type requires changes in 10 files
public class PriceCalculator {
    if (customer.type == "PREMIUM") // Change here
}

public class InvoiceGenerator {
    if (customer.type == "PREMIUM") // And here
}

public class EmailService {
    if (customer.type == "PREMIUM") // And here
}
// ... 7 more files
```

**Fix:** Move Function/Field to consolidate changes.

```java
// GOOD - Changes localized to one place
public class Customer {
    public double getDiscount() {
        if (type == CustomerType.PREMIUM)
            return 0.15;
        return 0.0;
    }
}

// All other classes just call customer.getDiscount()
```

**Source:** Clean Code, Refactoring

---

### Hidden Temporal Coupling

**Smell:** Methods must be called in order but nothing enforces it.

```java
// BAD
public class MoogDiver {
    public void dive(String reason) {
        saturateGradient();  // Must be called first!
        reticulateSplines();  // Must be called second!
        diveForMoog(reason);  // Must be called third!
    }
}
```

**Fix:** Bucket Brigade Pattern - each method returns input for next.

```java
// GOOD
public class MoogDiver {
    public void dive(String reason) {
        Gradient gradient = saturateGradient();
        List<Spline> splines = reticulateSplines(gradient);
        diveForMoog(splines, reason);
    }
}
```

**Source:** Clean Code

---

### Feature Envy (Coupling Issue)

*See [Feature Envy](#feature-envy) in Class Design section*

---

## Legacy Code Patterns

### Code Without Tests

**Smell:** Production code with no automated tests.

```java
// BAD - No tests
public class PaymentProcessor {
    public void processPayment(Order order) {
        // 200 lines of complex logic
        // No tests mean changes are risky
    }
}
```

**Fix:** Add Characterization Tests to document current behavior.

```java
// GOOD - Document behavior with tests
@Test
public void testProcessPayment_standardOrder() {
    PaymentProcessor processor = new PaymentProcessor();
    Order order = new Order(100.00);

    // Don't know what it should return, so guess:
    assertEquals(100.00, processor.processPayment(order));
    // Test fails: Expected: 100.00, Actual: 105.00
    // Now we know it adds 5% fee!
}

@Test
public void testProcessPayment_addsProcessingFee() {
    PaymentProcessor processor = new PaymentProcessor();
    Order order = new Order(100.00);

    assertEquals(105.00, processor.processPayment(order));  // Documents actual behavior
}
```

**Process:**
1. Write test with guessed expectation
2. Run it and let it fail
3. Use failure message to see actual behavior
4. Update test to document actual behavior
5. Now you have a safety net for changes

**Source:** Working Effectively with Legacy Code

---

### Hidden Dependencies

**Smell:** Dependencies created inside constructor or method.

```java
// BAD
public class MailingListDispatcher {
    private MailService service;

    public MailingListDispatcher() {
        service = new MailService();  // HIDDEN DEPENDENCY!
    }
}
```

**Fix:** Parameterize Constructor.

```java
// GOOD
public class MailingListDispatcher {
    private MailService service;

    // For production
    public MailingListDispatcher() {
        this(new MailService());
    }

    // For testing
    public MailingListDispatcher(MailService service) {
        this.service = service;
    }
}
```

**Source:** Working Effectively with Legacy Code, Code That Fits in Your Head

---

### Singleton Dependencies

**Smell:** Global singleton prevents testing.

```java
// BAD
public class Scheduler {
    public void schedule(Task task) {
        PermitRepository repository = PermitRepository.getInstance();
        Permit permit = repository.findPermit(task);
        // Hard to test with real singleton
    }
}
```

**Fix:** Introduce Static Setter for testing.

```java
// GOOD
public class PermitRepository {
    private static PermitRepository instance;

    public static PermitRepository getInstance() {
        if (instance == null) {
            instance = new PermitRepository();
        }
        return instance;
    }

    // For testing
    public static void setTestingInstance(PermitRepository newInstance) {
        instance = newInstance;
    }
}

// In test
@Before
public void setUp() {
    PermitRepository.setTestingInstance(new FakePermitRepository());
}

@After
public void tearDown() {
    PermitRepository.setTestingInstance(null);
}
```

**Source:** Working Effectively with Legacy Code

---

### Untestable Static Calls

**Smell:** Static method calls that can't be replaced.

```cpp
// BAD
bool CAsyncSslRec::Init() {
    if (!m_bFailureSent) {
        m_bFailureSent = TRUE;
        PostReceiveError(SOCKETCALLBACK, SSL_FAILURE);  // Static call
    }
    return true;
}
```

**Fix:** Extract and Override Method.

```cpp
// GOOD
class CAsyncSslRec {
protected:
    virtual void PostReceiveError(UINT type, UINT errorcode) {
        ::PostReceiveError(type, errorcode);  // Call global function
    }
};

// Test subclass
class TestingAsyncSslRec : public CAsyncSslRec {
protected:
    virtual void PostReceiveError(UINT type, UINT errorcode) {
        // Do nothing or record the call
    }
};
```

**Source:** Working Effectively with Legacy Code

---

### Monster Methods

**Smell:** Very long methods (100+ lines) with complex logic.

```java
// BAD
public void processTransaction(Transaction t) {
    // 150 lines of complex logic
    // Multiple responsibilities
    // Deep nesting
}
```

**Fix:** Break Out Method Object.

```java
// GOOD
public class TransactionProcessor {
    private Transaction transaction;
    private double total;
    private List<Item> items;

    public TransactionProcessor(Transaction t) {
        this.transaction = t;
    }

    public void process() {
        loadItems();
        calculateTotals();
        applyDiscounts();
        recordResults();
    }

    private void loadItems() { /* ... */ }
    private void calculateTotals() { /* ... */ }
    private void applyDiscounts() { /* ... */ }
    private void recordResults() { /* ... */ }
}
```

**Source:** Working Effectively with Legacy Code, Clean Code

---

### Irritating Parameters

**Smell:** Constructor requires complex objects hard to create in tests.

```java
// BAD
public class CreditValidator {
    public CreditValidator(RGHConnection connection,
                          CreditMaster master,
                          String validatorID) {
        // Hard to create these objects in tests
    }
}
```

**Fix Option 1:** Pass Null (if parameter not needed for test).

```java
@Test
public void testValidation() {
    CreditValidator validator = new CreditValidator(null, null, "1");
    // Test the parts that don't need the dependencies
}
```

**Fix Option 2:** Extract Interface.

```java
// GOOD
public interface Connection {
    void send(String message);
}

public class CreditValidator {
    public CreditValidator(Connection connection, ...) {
        // Now we can pass a fake Connection
    }
}

// In test
public class FakeConnection implements Connection {
    public void send(String message) {}
}
```

**Source:** Working Effectively with Legacy Code

---

## Comments

### Comments Explaining What Code Does

**Smell:** Comments describing what obvious code does.

```java
// BAD
// Check to see if the employee is eligible for full benefits
if ((employee.flags & HOURLY_FLAG) && (employee.age > 65))

// Noise comments
/**
 * Default constructor.
 */
protected AnnualDateRule() {
}
```

**Fix:** Extract Function or Rename to make code self-explanatory.

```java
// GOOD
if (employee.isEligibleForFullBenefits())

private boolean isEligibleForFullBenefits() {
    return ((employee.flags & HOURLY_FLAG) && (employee.age > 65));
}
```

**Acceptable comments:**
- Legal comments (copyright, licenses)
- Explanation of intent ("we're doing X because Y")
- Warning of consequences ("This takes 10 minutes to run")
- TODO comments (with ticket references)
- Public API documentation

**Source:** Clean Code, Refactoring

---

### Commented-Out Code

**Smell:** Code left commented "just in case."

```java
// BAD - just delete it!
public void process() {
    // InputStream resultsStream = formatter.getResultStream();
    // StreamReader reader = new StreamReader(resultsStream);
    // response.setContent(reader.read(formatter.getByteCount()));

    response.setBody(formatter.getResultStream(), formatter.getByteCount());
}
```

**Fix:** Delete it. Version control remembers it.

```java
// GOOD
public void process() {
    response.setBody(formatter.getResultStream(), formatter.getByteCount());
}
```

**Source:** Clean Code, Refactoring, Code That Fits in Your Head

---

### Misleading or Obsolete Comments

**Smell:** Comments that don't match the code.

```java
// BAD
// Check if user is admin
if (user.role == 'moderator') {  // Comment is wrong!
    // ...
}
```

**Fix:** Update or delete the comment. Better: make code self-explanatory.

```java
// GOOD
if (user.isModerator()) {  // No comment needed
    // ...
}
```

**Source:** Clean Code

---

## Duplication

### Duplicated Code

**Smell:** Same code structure in multiple places.

```java
// BAD
public void scaleToOneDimension(...) {
    RenderedOp newImage = ImageUtilities.getScaledImage(
        image, scalingFactor, scalingFactor
    );
    image.dispose();
    System.gc();
    image = newImage;
}

public synchronized void rotate(int degrees) {
    RenderedOp newImage = ImageUtilities.getRotatedImage(image, degrees);
    image.dispose();
    System.gc();
    image = newImage;
}
```

**Fix:** Extract Function.

```java
// GOOD
private void replaceImage(RenderedOp newImage) {
    image.dispose();
    System.gc();
    image = newImage;
}

public void scaleToOneDimension(...) {
    RenderedOp newImage = ImageUtilities.getScaledImage(
        image, scalingFactor, scalingFactor
    );
    replaceImage(newImage);
}

public synchronized void rotate(int degrees) {
    RenderedOp newImage = ImageUtilities.getRotatedImage(image, degrees);
    replaceImage(newImage);
}
```

**Source:** Clean Code, Refactoring

**Principle:** "Code should say everything once and only once."

---

## Speculative Generality

### Speculative Generality / YAGNI Violation

**Smell:** Hooks and abstractions for things that "might be needed someday."

```java
// BAD
public abstract class AbstractShapeFactory {
    // Unused abstract methods
    protected abstract void initialize();
    protected abstract void shutdown();
}

public interface Plugin {
    // Only one implementation exists
}

public void process(Strategy strategy) {
    // Only ever called with one strategy
}
```

**Fix:** Remove unused abstractions.

```java
// GOOD
public class ShapeFactory {
    // Concrete class with just what's needed
}

// Remove plugin interface, use concrete class

public void process() {
    // Remove strategy pattern if not needed
}
```

**Techniques:**
- Collapse Hierarchy (for unused abstractions)
- Inline Function/Class (remove unnecessary delegation)
- Change Function Declaration (remove unused parameters)
- Remove Dead Code (for functions/classes only used by tests)

**Source:** Clean Code, Refactoring, Code That Fits in Your Head

**Principle:** YAGNI - "You Aren't Going to Need It"

---

## Lazy Elements

### Lazy Element

**Smell:** Structure that doesn't pull its weight.

```java
// BAD
public int getTotalPrice() {
    return calculatePrice();
}

private int calculatePrice() {
    return basePrice * quantity;
}

// Class that's just one simple method
public class SimpleCalculator {
    public int add(int a, int b) {
        return a + b;
    }
}
```

**Fix:** Inline Function or Inline Class.

```java
// GOOD
public int getTotalPrice() {
    return basePrice * quantity;
}

// Just use a function or method in existing class
public int add(int a, int b) {
    return a + b;
}
```

**Source:** Refactoring

---

## Summary: Quick Reference

### Top 10 Most Critical Code Smells

1. **Long Functions** → Extract Function
2. **Duplicated Code** → Extract Function, Pull Up Method
3. **Large Classes** → Extract Class
4. **Long Parameter Lists** → Introduce Parameter Object
5. **Repeated Switches** → Replace Conditional with Polymorphism
6. **Feature Envy** → Move Function
7. **Global/Mutable Data** → Encapsulate Variable
8. **Testing Implementation** → Test Behavior Instead
9. **Hidden Dependencies** → Parameterize Constructor
10. **Primitive Obsession** → Replace Primitive with Object

### Complexity Metrics

- **Cyclomatic Complexity:** ≤ 7
- **Function Length:** ≤ 24 lines
- **Line Width:** ≤ 80 characters
- **Variables in Scope:** ≤ 7
- **Parameters:** ≤ 3

### Core Principles

1. **DRY** - Don't Repeat Yourself
2. **SOLID** - Single Responsibility, Open-Closed, Liskov Substitution, Interface Segregation, Dependency Inversion
3. **YAGNI** - You Aren't Going to Need It
4. **KISS** - Keep It Simple, Stupid
5. **Law of Demeter** - Talk to friends, not strangers
6. **Command-Query Separation** - Do or answer, not both
7. **Parse, Don't Validate** - Make invalid states unrepresentable
8. **Test Behavior, Not Implementation**
9. **One Level of Abstraction per Function**
10. **Boy Scout Rule** - Leave code cleaner than you found it

---

*This comprehensive reference synthesizes code smells and anti-patterns from five foundational software engineering books. Use it as a checklist during code reviews and refactoring sessions.*
